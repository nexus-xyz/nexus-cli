//! Online Workers
//!
//! Handles network-dependent operations including:
//! - Task fetching from the orchestrator
//! - Proof submission to the orchestrator
//! - Network error handling with exponential backoff

use crate::analytics::{
    track_got_task, track_proof_accepted, track_proof_submission_error,
    track_proof_submission_success,
};
use crate::consts::prover::{
    BACKOFF_DURATION, BATCH_SIZE, LOW_WATER_MARK, MAX_404S_BEFORE_GIVING_UP, QUEUE_LOG_INTERVAL,
    TASK_QUEUE_SIZE,
};
use crate::environment::Environment;
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
            backoff_duration: Duration::from_millis(BACKOFF_DURATION), // Start with 120 second backoff
            last_queue_log_time: std::time::Instant::now(),
            queue_log_interval: Duration::from_millis(QUEUE_LOG_INTERVAL), // Log queue status every 30 seconds
            error_classifier: ErrorClassifier::new(),
        }
    }

    pub fn should_log_queue_status(&mut self) -> bool {
        // Log queue status every QUEUE_LOG_INTERVAL seconds regardless of queue level
        self.last_queue_log_time.elapsed() >= self.queue_log_interval
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

    /// Set backoff duration from server's Retry-After header (in seconds)
    pub fn set_backoff_from_server(&mut self, retry_after_seconds: u32) {
        // Use the server's exact retry time
        self.backoff_duration = Duration::from_secs(retry_after_seconds as u64);
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
#[allow(clippy::too_many_arguments)]
pub async fn fetch_prover_tasks(
    node_id: u64,
    verifying_key: VerifyingKey,
    orchestrator_client: Box<dyn Orchestrator>,
    sender: mpsc::Sender<Task>,
    event_sender: mpsc::Sender<Event>,
    mut shutdown: broadcast::Receiver<()>,
    recent_tasks: TaskCache,
    environment: Environment,
    client_id: String,
) {
    let mut state = TaskFetchState::new();

    loop {
        tokio::select! {
            _ = shutdown.recv() => break,
            _ = tokio::time::sleep(Duration::from_millis(500)) => {
                let tasks_in_queue = TASK_QUEUE_SIZE - sender.capacity();

                // Log queue status every QUEUE_LOG_INTERVAL seconds regardless of queue level
                if state.should_log_queue_status() {
                    state.record_queue_log();
                    log_queue_status(&event_sender, tasks_in_queue, &state).await;
                }

                // Attempt fetch if conditions are met
                if state.should_fetch(tasks_in_queue) {
                    if let Err(should_return) = attempt_task_fetch(
                        &*orchestrator_client,
                        &node_id,
                        verifying_key,
                        &sender,
                        &event_sender,
                        &recent_tasks,
                        &mut state,
                        &environment,
                        &client_id,
                    ).await {
                        if should_return {
                            return;
                        }
                    }
                }
            }
        }
    }
}

/// Attempt to fetch tasks with timeout and error handling
#[allow(clippy::too_many_arguments)]
async fn attempt_task_fetch(
    orchestrator_client: &dyn Orchestrator,
    node_id: &u64,
    verifying_key: VerifyingKey,
    sender: &mpsc::Sender<Task>,
    event_sender: &mpsc::Sender<Event>,
    recent_tasks: &TaskCache,
    state: &mut TaskFetchState,
    environment: &Environment,
    client_id: &str,
) -> Result<(), bool> {
    let _ = event_sender
        .send(Event::task_fetcher_with_level(
            "[Task step 1 of 3] Fetching tasks...Note: CLI tasks are harder to solve, so they receive 10 times more points than web provers".to_string(),
            crate::events::EventType::Refresh,
            LogLevel::Debug,
        ))
        .await;

    // Add timeout to prevent hanging
    let fetch_future = fetch_task_batch(
        orchestrator_client,
        node_id,
        verifying_key,
        BATCH_SIZE,
        event_sender,
    );
    let timeout_duration = Duration::from_secs(60); // 60 second timeout

    match tokio::time::timeout(timeout_duration, fetch_future).await {
        Ok(fetch_result) => match fetch_result {
            Ok(tasks) => {
                // Record successful fetch attempt timing
                state.record_fetch_attempt();
                handle_fetch_success(
                    tasks,
                    sender,
                    event_sender,
                    recent_tasks,
                    state,
                    environment,
                    client_id,
                )
                .await
            }
            Err(e) => {
                // Record failed fetch attempt timing
                state.record_fetch_attempt();
                handle_fetch_error(e, event_sender, state).await;
                Ok(())
            }
        },
        Err(_timeout) => {
            // Handle timeout case
            state.record_fetch_attempt();
            let _ = event_sender
                .send(Event::task_fetcher_with_level(
                    format!("Fetch timeout after {}s", timeout_duration.as_secs()),
                    crate::events::EventType::Error,
                    LogLevel::Warn,
                ))
                .await;
            // Increase backoff for timeout
            state.increase_backoff_for_error();
            Ok(())
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

    let message = if state.should_fetch(tasks_in_queue) {
        format!(
            "Tasks Queue low: {} tasks to compute, ready to fetch",
            tasks_in_queue
        )
    } else {
        let time_since_secs = time_since_last.as_secs();
        format!(
            "Tasks to compute: {} tasks, waiting {}s more (retry every {}s)",
            tasks_in_queue,
            backoff_secs.saturating_sub(time_since_secs),
            backoff_secs
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
    environment: &Environment,
    client_id: &str,
) -> Result<(), bool> {
    if tasks.is_empty() {
        handle_empty_task_response(sender, event_sender, state).await;
        return Ok(());
    }

    let (added_count, duplicate_count) = process_fetched_tasks(
        tasks,
        sender,
        event_sender,
        recent_tasks,
        environment,
        client_id,
    )
    .await?;

    log_fetch_results(added_count, duplicate_count, sender, event_sender, state).await;
    Ok(())
}

/// Handle empty task response from server
async fn handle_empty_task_response(
    _sender: &mpsc::Sender<Task>,
    event_sender: &mpsc::Sender<Event>,
    state: &mut TaskFetchState,
) {
    let msg = "No tasks available yet for this node".to_string();
    let _ = event_sender
        .send(Event::task_fetcher_with_level(
            msg,
            crate::events::EventType::Refresh,
            LogLevel::Info,
        ))
        .await;

    // Only reset backoff if it's at or below default level
    // If backoff is higher, it's likely from a server retry-after header
    if state.backoff_duration <= Duration::from_millis(BACKOFF_DURATION) {
        state.reset_backoff();
    }
}

/// Process fetched tasks and handle duplicates
async fn process_fetched_tasks(
    tasks: Vec<Task>,
    sender: &mpsc::Sender<Task>,
    event_sender: &mpsc::Sender<Event>,
    recent_tasks: &TaskCache,
    environment: &Environment,
    client_id: &str,
) -> Result<(usize, usize), bool> {
    let mut added_count = 0;
    let mut duplicate_count = 0;

    for task in tasks {
        if recent_tasks.contains(&task.task_id).await {
            duplicate_count += 1;
            continue;
        }
        recent_tasks.insert(task.task_id.clone()).await;

        if sender.send(task.clone()).await.is_err() {
            let _ = event_sender
                .send(Event::task_fetcher(
                    "Task queue is closed".to_string(),
                    crate::events::EventType::Shutdown,
                ))
                .await;
            return Err(true); // Signal caller to return
        }

        // Track analytics for getting a task (non-blocking)
        tokio::spawn(track_got_task(
            task.clone(),
            environment.clone(),
            client_id.to_string(),
        ));

        added_count += 1;
    }

    Ok((added_count, duplicate_count))
}

/// Log fetch results and handle backoff logic
async fn log_fetch_results(
    added_count: usize,
    duplicate_count: usize,
    sender: &mpsc::Sender<Task>,
    event_sender: &mpsc::Sender<Event>,
    state: &mut TaskFetchState,
) {
    if added_count > 0 {
        log_successful_fetch(added_count, sender, event_sender).await;
        state.reset_backoff(); // Reset to default 120s backoff
    } else if duplicate_count > 0 {
        handle_all_duplicates(duplicate_count, event_sender, state).await;
    }
}

/// Log successful task fetch with queue status
async fn log_successful_fetch(
    added_count: usize,
    sender: &mpsc::Sender<Task>,
    event_sender: &mpsc::Sender<Event>,
) {
    let current_queue_level = TASK_QUEUE_SIZE - sender.capacity();
    let queue_percentage = (current_queue_level as f64 / TASK_QUEUE_SIZE as f64 * 100.0) as u32;

    // Enhanced queue status logging
    let msg = if added_count >= 5 {
        format!(
            "Queue status: +{} tasks → {} total ({}/{}={queued_percentage}% full)",
            added_count,
            current_queue_level,
            current_queue_level,
            TASK_QUEUE_SIZE,
            queued_percentage = queue_percentage
        )
    } else {
        format!(
            "Queue status: +{} tasks → {} total ({}% full)",
            added_count, current_queue_level, queue_percentage
        )
    };

    // Log level based on queue fullness
    let log_level = if queue_percentage >= 80 || added_count >= 5 {
        LogLevel::Info // High queue level or significant additions are important
    } else {
        LogLevel::Debug // Minor additions are debug level
    };

    let _ = event_sender
        .send(Event::task_fetcher_with_level(
            msg,
            crate::events::EventType::Refresh,
            log_level,
        ))
        .await;
}

/// Handle case where all fetched tasks were duplicates
async fn handle_all_duplicates(
    duplicate_count: usize,
    event_sender: &mpsc::Sender<Event>,
    state: &mut TaskFetchState,
) {
    // All duplicates - significant backoff increase
    state.increase_backoff_for_error();
    let _ = event_sender
        .send(Event::task_fetcher_with_level(
            format!(
                "All {} tasks were duplicates - backing off for {}s",
                duplicate_count,
                state.backoff_duration.as_secs()
            ),
            crate::events::EventType::Refresh,
            LogLevel::Warn,
        ))
        .await;
}

/// Handle fetch errors with appropriate backoff
async fn handle_fetch_error(
    error: OrchestratorError,
    event_sender: &mpsc::Sender<Event>,
    state: &mut TaskFetchState,
) {
    match error {
        OrchestratorError::Http {
            status: 429,
            ref headers,
            ..
        } => {
            // Debug: print headers for 429 responses
            let _ = event_sender
                .send(Event::task_fetcher_with_level(
                    format!("429 Rate limit retry-after: {:?}", headers["retry-after"]),
                    crate::events::EventType::Refresh,
                    LogLevel::Debug,
                ))
                .await;

            if let Some(retry_after_seconds) = error.get_retry_after_seconds() {
                state.set_backoff_from_server(retry_after_seconds);
                let _ = event_sender
                    .send(Event::task_fetcher_with_level(
                        format!("Rate limited - retrying in {}s", retry_after_seconds),
                        crate::events::EventType::Error,
                        LogLevel::Warn,
                    ))
                    .await;
            } else {
                // This shouldn't happen with a properly configured server
                state.increase_backoff_for_error();
                let _ = event_sender
                    .send(Event::task_fetcher_with_level(
                        "Rate limited - no retry time specified".to_string(),
                        crate::events::EventType::Error,
                        LogLevel::Error,
                    ))
                    .await;
            }
        }
        _ => {
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
}

/// Fetch a batch of tasks from the orchestrator
async fn fetch_task_batch(
    orchestrator_client: &dyn Orchestrator,
    node_id: &u64,
    verifying_key: VerifyingKey,
    batch_size: usize,
    event_sender: &mpsc::Sender<Event>,
) -> Result<Vec<Task>, OrchestratorError> {
    // First try to get existing assigned tasks
    if let Some(existing_tasks) = try_get_existing_tasks(orchestrator_client, node_id).await? {
        return Ok(existing_tasks);
    }

    // If no existing tasks, try to get new ones
    fetch_new_tasks_batch(
        orchestrator_client,
        node_id,
        verifying_key,
        batch_size,
        event_sender,
    )
    .await
}

/// Try to get existing assigned tasks
async fn try_get_existing_tasks(
    orchestrator_client: &dyn Orchestrator,
    node_id: &u64,
) -> Result<Option<Vec<Task>>, OrchestratorError> {
    match orchestrator_client.get_tasks(&node_id.to_string()).await {
        Ok(tasks) => {
            if !tasks.is_empty() {
                Ok(Some(tasks))
            } else {
                Ok(None)
            }
        }
        Err(e) => {
            // If getting existing tasks fails, try to get new ones
            if matches!(e, OrchestratorError::Http { status: 404, .. }) {
                Ok(None)
            } else {
                Err(e)
            }
        }
    }
}

/// Fetch a batch of new tasks from the orchestrator
async fn fetch_new_tasks_batch(
    orchestrator_client: &dyn Orchestrator,
    node_id: &u64,
    verifying_key: VerifyingKey,
    batch_size: usize,
    event_sender: &mpsc::Sender<Event>,
) -> Result<Vec<Task>, OrchestratorError> {
    let mut new_tasks = Vec::new();
    let mut consecutive_404s = 0;

    for i in 0..batch_size {
        match orchestrator_client
            .get_proof_task(&node_id.to_string(), verifying_key)
            .await
        {
            Ok(task) => {
                new_tasks.push(task);
                consecutive_404s = 0; // Reset counter on success
            }
            Err(OrchestratorError::Http {
                status: 429,
                message,
                ref headers,
            }) => {
                // Debug: print headers for 429 responses
                let _ = event_sender
                    .send(Event::task_fetcher_with_level(
                        "Every node in the Prover Network is rate limited to 3 tasks per 3 minutes"
                            .to_string(),
                        crate::events::EventType::Refresh,
                        LogLevel::Debug,
                    ))
                    .await;

                // Don't handle 429 here - propagate it back to main error handler
                return Err(OrchestratorError::Http {
                    status: 429,
                    message,
                    headers: headers.clone(),
                });
            }
            Err(OrchestratorError::Http { status: 404, .. }) => {
                consecutive_404s += 1;
                let _ = event_sender
                    .send(Event::task_fetcher_with_level(
                        format!("fetch_task_batch: No task available (404) on attempt #{}, consecutive_404s: {}", i + 1, consecutive_404s),
                        crate::events::EventType::Refresh,
                        LogLevel::Debug,
                    ))
                    .await;

                if consecutive_404s >= MAX_404S_BEFORE_GIVING_UP {
                    let _ = event_sender
                        .send(Event::task_fetcher_with_level(
                            format!(
                                "fetch_task_batch: Too many 404s ({}), giving up",
                                consecutive_404s
                            ),
                            crate::events::EventType::Refresh,
                            LogLevel::Debug,
                        ))
                        .await;
                    break;
                }
                // Continue trying more tasks
            }
            Err(e) => {
                let _ = event_sender
                    .send(Event::task_fetcher_with_level(
                        format!(
                            "fetch_task_batch: get_proof_task #{} failed with error: {:?}",
                            i + 1,
                            e
                        ),
                        crate::events::EventType::Refresh,
                        LogLevel::Debug,
                    ))
                    .await;
                return Err(e);
            }
        }
    }

    Ok(new_tasks)
}

/// Submits proofs to the orchestrator
#[allow(clippy::too_many_arguments)]
pub async fn submit_proofs(
    signing_key: SigningKey,
    orchestrator: Box<dyn Orchestrator>,
    num_workers: usize,
    mut results: mpsc::Receiver<(Task, Proof)>,
    event_sender: mpsc::Sender<Event>,
    mut shutdown: broadcast::Receiver<()>,
    successful_tasks: TaskCache,
    environment: Environment,
    client_id: String,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut completed_count = 0;
        let mut last_stats_time = std::time::Instant::now();
        let stats_interval = Duration::from_secs(60);

        loop {
            tokio::select! {
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
                                &environment,
                                &client_id,
                            ).await {
                                if success {
                                    completed_count += 1;
                                }
                            }

                            // Check if it's time to report stats (avoid timer starvation)
                            if last_stats_time.elapsed() >= stats_interval {
                                report_performance_stats(&event_sender, completed_count, last_stats_time).await;
                                completed_count = 0;
                                last_stats_time = std::time::Instant::now();
                            }
                        }
                        None => break,
                    }
                }

                _ = tokio::time::sleep(stats_interval) => {
                    // Fallback timer in case there's no activity
                    report_performance_stats(&event_sender, completed_count, last_stats_time).await;
                    completed_count = 0;
                    last_stats_time = std::time::Instant::now();
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
        "Performance Status: {} tasks completed in the past {:.1}s ({:.1} tasks/min)",
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
#[allow(clippy::too_many_arguments)]
async fn process_proof_submission(
    task: Task,
    proof: Proof,
    orchestrator: &dyn Orchestrator,
    signing_key: &SigningKey,
    num_workers: usize,
    event_sender: &mpsc::Sender<Event>,
    successful_tasks: &TaskCache,
    environment: &Environment,
    client_id: &str,
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
            task.task_type,
        )
        .await
    {
        Ok(_) => {
            // Track analytics for proof submission success (non-blocking)
            tokio::spawn(track_proof_submission_success(
                task.clone(),
                environment.clone(),
                client_id.to_string(),
            ));
            handle_submission_success(
                &task,
                event_sender,
                successful_tasks,
                environment,
                client_id,
            )
            .await;
            Some(true)
        }
        Err(e) => {
            handle_submission_error(&task, e, event_sender, environment, client_id).await;
            Some(false)
        }
    }
}

/// Handle successful proof submission
async fn handle_submission_success(
    task: &Task,
    event_sender: &mpsc::Sender<Event>,
    successful_tasks: &TaskCache,
    environment: &Environment,
    client_id: &str,
) {
    successful_tasks.insert(task.task_id.clone()).await;
    let msg = format!(
        "[Task step 3 of 3] Proof submitted (Task ID: {}) Points for this node will be updated in https://app.nexus.xyz/rewards within 10 minutes",
        task.task_id
    );
    // Track analytics for proof acceptance (non-blocking)
    tokio::spawn(track_proof_accepted(
        task.clone(),
        environment.clone(),
        client_id.to_string(),
    ));

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
    environment: &Environment,
    client_id: &str,
) {
    let (msg, status_code) = match error {
        OrchestratorError::Http { status, .. } => (
            format!(
                "Failed to submit proof for task {}. Status: {}",
                task.task_id, status
            ),
            Some(status),
        ),
        e => (
            format!("Failed to submit proof for task {}: {}", task.task_id, e),
            None,
        ),
    };

    // Track analytics for proof submission error (non-blocking)
    tokio::spawn(track_proof_submission_error(
        task.clone(),
        msg.clone(),
        status_code,
        environment.clone(),
        client_id.to_string(),
    ));

    let _ = event_sender
        .send(Event::proof_submitter(
            msg.to_string(),
            crate::events::EventType::Error,
        ))
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_set_backoff_from_server() {
        let mut state = TaskFetchState::new();

        // Test setting a reasonable retry time
        state.set_backoff_from_server(60);
        assert_eq!(state.backoff_duration, Duration::from_secs(60));

        // Test that longer retry times are respected (no capping)
        state.set_backoff_from_server(300); // 5 minutes
        assert_eq!(state.backoff_duration, Duration::from_secs(300));

        // Test zero retry time
        state.set_backoff_from_server(0);
        assert_eq!(state.backoff_duration, Duration::from_secs(0));
    }

    #[test]
    fn test_server_retry_times_respected() {
        let mut state = TaskFetchState::new();

        // Test that very long retry times are respected
        state.set_backoff_from_server(3600); // 1 hour
        assert_eq!(state.backoff_duration, Duration::from_secs(3600));
    }

    #[test]
    fn test_reset_backoff() {
        let mut state = TaskFetchState::new();

        // Test that reset sets backoff to default 120s
        state.reset_backoff();
        assert_eq!(
            state.backoff_duration,
            Duration::from_millis(BACKOFF_DURATION)
        );
    }
}
