//! Online Workers
//!
//! Handles network-dependent operations including:
//! - Task fetching from the orchestrator
//! - Proof submission to the orchestrator
//! - Network error handling with exponential backoff

use crate::error_classifier::{ErrorClassifier, LogLevel};
use crate::events::Event;
use crate::orchestrator::Orchestrator;
use crate::orchestrator::error::OrchestratorError;
use crate::task::Task;
use crate::task_cache::TaskCache;
use ed25519_dalek::{SigningKey, VerifyingKey};
use nexus_sdk::stwo::seq::Proof;
use sha3::{Digest, Keccak256};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

// Task fetching thresholds
const BATCH_SIZE: usize = 50; // Fetch this many tasks at once
const LOW_WATER_MARK: usize = BATCH_SIZE / 5; // Fetch new tasks when queue drops below this
const TASK_QUEUE_SIZE: usize = 100; // Queue sizes from runtime
const MAX_404S_BEFORE_GIVING_UP: usize = 5; // Allow several 404s before stopping batch fetch
const BACKOFF_DURATION: u64 = 30000; // 30 seconds
const QUEUE_LOG_INTERVAL: u64 = 30000; // 30 seconds

/// State for managing task fetching behavior
pub struct TaskFetchState {
    last_fetch_time: std::time::Instant,
    backoff_duration: Duration,
    last_queue_log_time: std::time::Instant,
    queue_log_interval: Duration,
    error_classifier: ErrorClassifier,
}

impl TaskFetchState {
    pub fn new() -> Self {
        Self {
            last_fetch_time: std::time::Instant::now()
                - Duration::from_millis(BACKOFF_DURATION + 1000), // Allow immediate first fetch
            backoff_duration: Duration::from_millis(BACKOFF_DURATION), // Start with 30 second backoff
            last_queue_log_time: std::time::Instant::now(),
            queue_log_interval: Duration::from_millis(QUEUE_LOG_INTERVAL), // Log queue status every 30 seconds
            error_classifier: ErrorClassifier::new(),
        }
    }

    pub fn should_log_queue_status(&mut self, tasks_in_queue: usize) -> bool {
        tasks_in_queue < LOW_WATER_MARK
            && self.last_queue_log_time.elapsed() >= self.queue_log_interval
    }

    pub fn should_fetch(&self, tasks_in_queue: usize) -> bool {
        tasks_in_queue < LOW_WATER_MARK && self.last_fetch_time.elapsed() >= self.backoff_duration
    }

    pub fn record_fetch_attempt(&mut self) {
        self.last_fetch_time = std::time::Instant::now();
    }

    pub fn record_queue_log(&mut self) {
        self.last_queue_log_time = std::time::Instant::now();
    }

    pub fn reset_backoff(&mut self) {
        self.backoff_duration = Duration::from_millis(BACKOFF_DURATION);
    }

    pub fn increase_backoff_for_rate_limit(&mut self) {
        self.backoff_duration = std::cmp::min(
            self.backoff_duration * 2,
            Duration::from_millis(BACKOFF_DURATION * 2),
        );
    }

    pub fn increase_backoff_for_error(&mut self) {
        self.backoff_duration = std::cmp::min(
            self.backoff_duration * 2,
            Duration::from_millis(BACKOFF_DURATION * 2),
        );
    }
}

/// Fetches tasks from the orchestrator and place them in the task queue.
/// Uses demand-driven fetching: only fetches when queue drops below LOW_WATER_MARK.
pub async fn fetch_prover_tasks(
    node_id: u64,
    verifying_key: VerifyingKey,
    orchestrator_client: Box<dyn Orchestrator>,
    sender: mpsc::Sender<Task>,
    event_sender: mpsc::Sender<Event>,
    mut shutdown: broadcast::Receiver<()>,
    recent_tasks: TaskCache,
) {
    let mut state = TaskFetchState::new();

    loop {
        tokio::select! {
            _ = shutdown.recv() => break,
            _ = tokio::time::sleep(Duration::from_millis(500)) => {
                let tasks_in_queue = TASK_QUEUE_SIZE - sender.capacity();

                // Log queue status occasionally
                if state.should_log_queue_status(tasks_in_queue) {
                    state.record_queue_log();
                    log_queue_status(&event_sender, tasks_in_queue, &state).await;
                }

                // Attempt fetch if conditions are met
                if state.should_fetch(tasks_in_queue) {
                    state.record_fetch_attempt();

                    match fetch_task_batch(&*orchestrator_client, &node_id, verifying_key, BATCH_SIZE).await {
                        Ok(tasks) => {
                            if let Err(should_return) = handle_fetch_success(
                                tasks,
                                &sender,
                                &event_sender,
                                &recent_tasks,
                                &mut state
                            ).await {
                                if should_return {
                                    return;
                                }
                            }
                        }
                        Err(e) => {
                            handle_fetch_error(e, &event_sender, &mut state).await;
                        }
                    }
                }
            }
        }
    }
}

/// Log the current queue status
async fn log_queue_status(
    event_sender: &mpsc::Sender<Event>,
    tasks_in_queue: usize,
    state: &TaskFetchState,
) {
    let time_since_last = state.last_fetch_time.elapsed();
    let backoff_secs = state.backoff_duration.as_secs();

    let message = if time_since_last >= state.backoff_duration {
        format!("Queue low: {} tasks, fetching now...", tasks_in_queue)
    } else {
        let time_since_secs = time_since_last.as_secs();
        format!(
            "Queue low: {} tasks, last fetch {}s ago (retry every {}s)",
            tasks_in_queue, time_since_secs, backoff_secs
        )
    };

    let _ = event_sender
        .send(Event::task_fetcher_with_level(
            message,
            crate::events::EventType::Refresh,
            LogLevel::Debug,
        ))
        .await;
}

/// Handle successful task fetch
async fn handle_fetch_success(
    tasks: Vec<Task>,
    sender: &mpsc::Sender<Task>,
    event_sender: &mpsc::Sender<Event>,
    recent_tasks: &TaskCache,
    state: &mut TaskFetchState,
) -> Result<(), bool> {
    // bool indicates if caller should return
    if tasks.is_empty() {
        let _ = event_sender
            .send(Event::task_fetcher_with_level(
                "No tasks available from server".to_string(),
                crate::events::EventType::Refresh,
                LogLevel::Info,
            ))
            .await;
        return Ok(());
    }

    let mut added_count = 0;
    for task in tasks {
        if recent_tasks.contains(&task.task_id).await {
            continue;
        }
        recent_tasks.insert(task.task_id.clone()).await;

        if sender.send(task).await.is_err() {
            let _ = event_sender
                .send(Event::task_fetcher(
                    "Task queue is closed".to_string(),
                    crate::events::EventType::Shutdown,
                ))
                .await;
            return Err(true); // Signal caller to return
        }
        added_count += 1;
    }

    if added_count > 0 {
        // Only log significant additions
        if added_count >= 5 {
            let msg = format!(
                "Added {} tasks to queue (queue level: {})",
                added_count,
                TASK_QUEUE_SIZE - sender.capacity()
            );
            let _ = event_sender
                .send(Event::task_fetcher_with_level(
                    msg,
                    crate::events::EventType::Refresh,
                    LogLevel::Info,
                ))
                .await;
        }
        state.reset_backoff();
    }

    Ok(())
}

/// Handle fetch errors with appropriate backoff
async fn handle_fetch_error(
    error: OrchestratorError,
    event_sender: &mpsc::Sender<Event>,
    state: &mut TaskFetchState,
) {
    if matches!(error, OrchestratorError::Http { status: 429, .. }) {
        state.increase_backoff_for_rate_limit();
        let _ = event_sender
            .send(Event::task_fetcher_with_level(
                format!(
                    "Rate limited (429), backing off for {} seconds",
                    state.backoff_duration.as_secs()
                ),
                crate::events::EventType::Error,
                LogLevel::Warn,
            ))
            .await;
    } else {
        state.increase_backoff_for_error();
        let log_level = state.error_classifier.classify_fetch_error(&error);
        let event = Event::task_fetcher_with_level(
            format!(
                "Failed to fetch tasks: {}, retrying in {} seconds",
                error,
                state.backoff_duration.as_secs()
            ),
            crate::events::EventType::Error,
            log_level,
        );
        if event.should_display() {
            let _ = event_sender.send(event).await;
        }
    }
}

/// Fetch a batch of tasks from the orchestrator
async fn fetch_task_batch(
    orchestrator_client: &dyn Orchestrator,
    node_id: &u64,
    verifying_key: VerifyingKey,
    batch_size: usize,
) -> Result<Vec<Task>, OrchestratorError> {
    // First try to get existing assigned tasks
    match orchestrator_client.get_tasks(&node_id.to_string()).await {
        Ok(tasks) => {
            if !tasks.is_empty() {
                return Ok(tasks);
            }
        }
        Err(e) => {
            // If getting existing tasks fails, try to get new ones
            if !matches!(e, OrchestratorError::Http { status: 404, .. }) {
                return Err(e);
            }
        }
    }

    // If no existing tasks, try to get new ones
    let mut new_tasks = Vec::new();
    let mut consecutive_404s = 0;

    for _ in 0..batch_size {
        match orchestrator_client
            .get_proof_task(&node_id.to_string(), verifying_key)
            .await
        {
            Ok(task) => {
                new_tasks.push(task);
                consecutive_404s = 0; // Reset counter on success
            }
            Err(OrchestratorError::Http { status: 429, .. }) => {
                // Rate limited, return what we have
                break;
            }
            Err(OrchestratorError::Http { status: 404, .. }) => {
                // No more tasks available - but don't give up immediately
                consecutive_404s += 1;
                if consecutive_404s >= MAX_404S_BEFORE_GIVING_UP {
                    break;
                }
                // Continue trying more tasks
            }
            Err(e) => return Err(e),
        }
    }

    Ok(new_tasks)
}

/// Submits proofs to the orchestrator
pub async fn submit_proofs(
    signing_key: SigningKey,
    orchestrator: Box<dyn Orchestrator>,
    num_workers: usize,
    mut results: mpsc::Receiver<(Task, Proof)>,
    event_sender: mpsc::Sender<Event>,
    mut shutdown: broadcast::Receiver<()>,
    successful_tasks: TaskCache,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut completed_count = 0;
        let mut last_stats_time = std::time::Instant::now();
        let stats_interval = Duration::from_secs(60);

        loop {
            tokio::select! {
                _ = tokio::time::sleep(stats_interval) => {
                    report_performance_stats(&event_sender, completed_count, last_stats_time).await;
                    completed_count = 0;
                    last_stats_time = std::time::Instant::now();
                }

                maybe_item = results.recv() => {
                    match maybe_item {
                        Some((task, proof)) => {
                            if let Some(success) = process_proof_submission(
                                task,
                                proof,
                                &*orchestrator,
                                &signing_key,
                                num_workers,
                                &event_sender,
                                &successful_tasks,
                            ).await {
                                if success {
                                    completed_count += 1;
                                }
                            }
                        }
                        None => break,
                    }
                }

                _ = shutdown.recv() => break,
            }
        }
    })
}

/// Report performance statistics
async fn report_performance_stats(
    event_sender: &mpsc::Sender<Event>,
    completed_count: u64,
    last_stats_time: std::time::Instant,
) {
    let elapsed = last_stats_time.elapsed();
    let tasks_per_minute = if elapsed.as_secs() > 0 {
        (completed_count as f64 * 60.0) / elapsed.as_secs() as f64
    } else {
        0.0
    };

    let msg = format!(
        "Performance: {} tasks completed in {:.1}s ({:.1} tasks/min)",
        completed_count,
        elapsed.as_secs_f64(),
        tasks_per_minute
    );
    let _ = event_sender
        .send(Event::proof_submitter_with_level(
            msg,
            crate::events::EventType::Refresh,
            LogLevel::Info,
        ))
        .await;
}

/// Process a single proof submission
/// Returns Some(true) if successful, Some(false) if failed, None if should skip
async fn process_proof_submission(
    task: Task,
    proof: Proof,
    orchestrator: &dyn Orchestrator,
    signing_key: &SigningKey,
    num_workers: usize,
    event_sender: &mpsc::Sender<Event>,
    successful_tasks: &TaskCache,
) -> Option<bool> {
    // Check for duplicate submissions
    if successful_tasks.contains(&task.task_id).await {
        let msg = format!(
            "Ignoring proof for previously submitted task {}",
            task.task_id
        );
        let _ = event_sender
            .send(Event::proof_submitter(msg, crate::events::EventType::Error))
            .await;
        return None; // Skip this task
    }

    // Serialize proof
    let proof_bytes = postcard::to_allocvec(&proof).expect("Failed to serialize proof");
    let proof_hash = format!("{:x}", Keccak256::digest(&proof_bytes));

    // Submit to orchestrator
    match orchestrator
        .submit_proof(
            &task.task_id,
            &proof_hash,
            proof_bytes,
            signing_key.clone(),
            num_workers,
        )
        .await
    {
        Ok(_) => {
            handle_submission_success(&task, event_sender, successful_tasks).await;
            Some(true)
        }
        Err(e) => {
            handle_submission_error(&task, e, event_sender).await;
            Some(false)
        }
    }
}

/// Handle successful proof submission
async fn handle_submission_success(
    task: &Task,
    event_sender: &mpsc::Sender<Event>,
    successful_tasks: &TaskCache,
) {
    successful_tasks.insert(task.task_id.clone()).await;
    let msg = format!("Successfully submitted proof for task {}", task.task_id);
    let _ = event_sender
        .send(Event::proof_submitter_with_level(
            msg,
            crate::events::EventType::Success,
            LogLevel::Info,
        ))
        .await;
}

/// Handle proof submission errors
async fn handle_submission_error(
    task: &Task,
    error: OrchestratorError,
    event_sender: &mpsc::Sender<Event>,
) {
    let msg = match error {
        OrchestratorError::Http { status, .. } => {
            format!(
                "Failed to submit proof for task {}. Status: {}",
                task.task_id, status
            )
        }
        e => {
            format!("Failed to submit proof for task {}: {}", task.task_id, e)
        }
    };

    let _ = event_sender
        .send(Event::proof_submitter(msg, crate::events::EventType::Error))
        .await;
}
