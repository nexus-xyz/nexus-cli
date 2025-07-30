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
    BACKOFF_DURATION, LOW_WATER_MARK, QUEUE_LOG_INTERVAL, TASK_QUEUE_SIZE,
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

/// Result of a proof generation, including combined hash for multiple inputs
pub struct ProofResult {
    pub proof: Proof,
    pub combined_hash: String,
}

/// Helper to send events with consistent error handling
async fn send_event(
    event_sender: &mpsc::Sender<Event>,
    message: String,
    event_type: crate::events::EventType,
    log_level: LogLevel,
) {
    let _ = event_sender
        .send(Event::task_fetcher_with_level(
            message, event_type, log_level,
        ))
        .await;
}

/// Helper to send proof submission events with consistent error handling
async fn send_proof_event(
    event_sender: &mpsc::Sender<Event>,
    message: String,
    event_type: crate::events::EventType,
    log_level: LogLevel,
) {
    let _ = event_sender
        .send(Event::proof_submitter_with_level(
            message, event_type, log_level,
        ))
        .await;
}

// =============================================================================
// TASK FETCH STATE
// =============================================================================

/// State for managing task fetching behavior with smart backoff and timing
pub struct TaskFetchState {
    last_fetch_time: std::time::Instant,
    backoff_duration: Duration,
    last_queue_log_time: std::time::Instant,
    queue_log_interval: Duration,
    pub error_classifier: ErrorClassifier,
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

    // =========================================================================
    // QUERY METHODS
    // =========================================================================

    /// Check if it's time to log queue status
    pub fn should_log_queue_status(&self) -> bool {
        self.last_queue_log_time.elapsed() >= self.queue_log_interval
    }

    /// Check if enough time has passed since last fetch attempt (respects backoff)
    pub fn can_fetch_now(&self) -> bool {
        self.last_fetch_time.elapsed() >= self.backoff_duration
    }

    /// Get current backoff duration
    pub fn backoff_duration(&self) -> Duration {
        self.backoff_duration
    }

    /// Get time since last fetch attempt
    pub fn time_since_last_fetch(&self) -> Duration {
        self.last_fetch_time.elapsed()
    }

    // =========================================================================
    // MUTATION METHODS
    // =========================================================================

    /// Record that a fetch attempt was made (updates timing)
    pub fn record_fetch_attempt(&mut self) {
        self.last_fetch_time = std::time::Instant::now();
    }

    /// Record that queue status was logged (updates timing)
    pub fn record_queue_log(&mut self) {
        self.last_queue_log_time = std::time::Instant::now();
    }

    /// Reset backoff to default duration (after successful operation)
    pub fn reset_backoff(&mut self) {
        self.backoff_duration = Duration::from_millis(BACKOFF_DURATION);
    }

    /// Set backoff duration from server's Retry-After header (in seconds)
    /// Respects server's exact timing for rate limit compliance
    pub fn set_backoff_from_server(&mut self, retry_after_seconds: u32) {
        self.backoff_duration = Duration::from_secs(retry_after_seconds as u64);
    }

    /// Increase backoff duration for error handling (exponential backoff)
    pub fn increase_backoff_for_error(&mut self) {
        self.backoff_duration = std::cmp::min(
            self.backoff_duration * 2,
            Duration::from_millis(BACKOFF_DURATION * 2),
        );
    }
}

/// Simple task fetcher: get one task at a time when queue is low.
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

                // Log queue status periodically
                if state.should_log_queue_status() {
                    state.record_queue_log();
                    log_queue_status(&event_sender, tasks_in_queue, &state).await;
                }

                // Simple condition: fetch when queue is low and backoff time has passed
                if tasks_in_queue < LOW_WATER_MARK && state.can_fetch_now() {
                    if let Err(should_return) = fetch_single_task_simple(
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

/// Simple task fetcher: get one task, prove, submit - perfect 1-2-3 flow
#[allow(clippy::too_many_arguments)]
async fn fetch_single_task_simple(
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
    // Record fetch attempt
    state.record_fetch_attempt();

    let _ = event_sender
        .send(Event::task_fetcher_with_level(
            "[Task step 1 of 3] Fetching task... Note: CLI tasks are harder to solve, so they receive 10 times more points than web provers".to_string(),
            crate::events::EventType::Refresh,
            LogLevel::Info,
        ))
        .await;

    // Fetch one task with timeout
    let timeout_duration = Duration::from_secs(60);
    let node_id_str = node_id.to_string();
    let fetch_future = orchestrator_client.get_proof_task(&node_id_str, verifying_key);

    match tokio::time::timeout(timeout_duration, fetch_future).await {
        Ok(Ok(task)) => {
            // Check for duplicate
            if recent_tasks.contains(&task.task_id).await {
                state.increase_backoff_for_error();
                send_event(
                    event_sender,
                    format!(
                        "Task was duplicate - backing off for {}s",
                        state.backoff_duration().as_secs()
                    ),
                    crate::events::EventType::Refresh,
                    LogLevel::Warn,
                )
                .await;
                return Ok(());
            }

            // Add to cache and queue
            recent_tasks.insert(task.task_id.clone()).await;

            if sender.send(task.clone()).await.is_err() {
                let _ = event_sender
                    .send(Event::task_fetcher(
                        "Task queue is closed".to_string(),
                        crate::events::EventType::Shutdown,
                    ))
                    .await;
                return Err(true); // Signal shutdown
            }

            // Track analytics
            tokio::spawn(track_got_task(
                task,
                environment.clone(),
                client_id.to_string(),
            ));

            // Success: reset backoff and log
            state.reset_backoff();
            let current_queue_level = TASK_QUEUE_SIZE - sender.capacity();
            let queue_percentage =
                (current_queue_level as f64 / TASK_QUEUE_SIZE as f64 * 100.0) as u32;
            let _ = event_sender
                .send(Event::task_fetcher_with_level(
                    format!(
                        "Queue status: +1 task → {} total ({}% full)",
                        current_queue_level, queue_percentage
                    ),
                    crate::events::EventType::Refresh,
                    if queue_percentage >= 80 {
                        LogLevel::Info
                    } else {
                        LogLevel::Debug
                    },
                ))
                .await;

            Ok(())
        }
        Ok(Err(e)) => {
            handle_fetch_error(e, event_sender, state).await;
            Ok(())
        }
        Err(_timeout) => {
            state.increase_backoff_for_error();
            send_event(
                event_sender,
                format!("Fetch timeout after {}s", timeout_duration.as_secs()),
                crate::events::EventType::Error,
                LogLevel::Warn,
            )
            .await;
            Ok(())
        }
    }
}

/// Log the current queue status with timing information
async fn log_queue_status(
    event_sender: &mpsc::Sender<Event>,
    tasks_in_queue: usize,
    state: &TaskFetchState,
) {
    let time_since_last = state.time_since_last_fetch();
    let backoff_duration = state.backoff_duration();
    let backoff_secs = backoff_duration.as_secs();

    let message = if tasks_in_queue < LOW_WATER_MARK && state.can_fetch_now() {
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

    send_event(
        event_sender,
        message,
        crate::events::EventType::Refresh,
        LogLevel::Debug,
    )
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
                send_event(
                    event_sender,
                    format!("Rate limited - retrying in {}s", retry_after_seconds),
                    crate::events::EventType::Error,
                    LogLevel::Warn,
                )
                .await;
            } else {
                // This shouldn't happen with a properly configured server
                state.increase_backoff_for_error();
                send_event(
                    event_sender,
                    "Rate limited - no retry time specified".to_string(),
                    crate::events::EventType::Error,
                    LogLevel::Error,
                )
                .await;
            }
        }
        _ => {
            state.increase_backoff_for_error();
            let log_level = state.error_classifier.classify_fetch_error(&error);
            let event = Event::task_fetcher_with_level(
                format!(
                    "Failed to fetch task: {}, retrying in {} seconds",
                    error,
                    state.backoff_duration().as_secs()
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

/// Submits proofs to the orchestrator
#[allow(clippy::too_many_arguments)]
pub async fn submit_proofs(
    signing_key: SigningKey,
    orchestrator: Box<dyn Orchestrator>,
    num_workers: usize,
    mut results: mpsc::Receiver<(Task, ProofResult)>,
    event_sender: mpsc::Sender<Event>,
    mut shutdown: broadcast::Receiver<()>,
    successful_tasks: TaskCache,
    environment: Environment,
    client_id: String,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                maybe_item = results.recv() => {
                    match maybe_item {
                        Some((task, proof_result)) => {
                            process_proof_submission(
                                task,
                                proof_result.proof,
                                proof_result.combined_hash,
                                &*orchestrator,
                                &signing_key,
                                num_workers,
                                &event_sender,
                                &successful_tasks,
                                &environment,
                                &client_id,
                            ).await;
                        }
                        None => break,
                    }
                }

                _ = shutdown.recv() => break,
            }
        }
    })
}

/// Process a single proof submission
#[allow(clippy::too_many_arguments)]
async fn process_proof_submission(
    task: Task,
    proof: Proof,
    combined_hash: String,
    orchestrator: &dyn Orchestrator,
    signing_key: &SigningKey,
    num_workers: usize,
    event_sender: &mpsc::Sender<Event>,
    successful_tasks: &TaskCache,
    environment: &Environment,
    client_id: &str,
) {
    // Check for duplicate submissions
    if successful_tasks.contains(&task.task_id).await {
        let msg = format!(
            "Ignoring proof for previously submitted task {}",
            task.task_id
        );
        let _ = event_sender
            .send(Event::proof_submitter(msg, crate::events::EventType::Error))
            .await;
        return; // Skip this task
    }

    // Use combined hash if provided, otherwise generate from proof
    let proof_hash = if !combined_hash.is_empty() {
        combined_hash
    } else {
        // Serialize proof and generate hash
        let proof_bytes = postcard::to_allocvec(&proof).expect("Failed to serialize proof");
        format!("{:x}", Keccak256::digest(&proof_bytes))
    };

    // Serialize proof for submission
    let proof_bytes = postcard::to_allocvec(&proof).expect("Failed to serialize proof");

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
        }
        Err(e) => {
            handle_submission_error(&task, e, event_sender, environment, client_id).await;
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

    send_proof_event(
        event_sender,
        msg,
        crate::events::EventType::Success,
        LogLevel::Info,
    )
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
        OrchestratorError::Http {
            status,
            ref message,
            ..
        } => (
            format!(
                "Failed to submit proof for task {}. Status: {}, Message: {}",
                task.task_id, status, message
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
