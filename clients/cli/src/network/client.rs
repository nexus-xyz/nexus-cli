//! Network client with built-in retry and error handling

use super::error_handler::ErrorHandler;
use super::request_timer::RequestTimer;
use crate::orchestrator::Orchestrator;
use crate::orchestrator::error::OrchestratorError;
use crate::task::Task;
use ed25519_dalek::{SigningKey, VerifyingKey};

use std::time::Duration;

/// Network client with built-in retry and request timing
pub struct NetworkClient {
    error_handler: ErrorHandler,
    request_timer: RequestTimer,
    max_retries: u32,
}

impl NetworkClient {
    pub fn new(request_timer: RequestTimer, max_retries: u32) -> Self {
        Self {
            error_handler: ErrorHandler::new(),
            request_timer,
            max_retries,
        }
    }

    /// Fetch a task with automatic retry and server-controlled timing
    pub async fn fetch_task(
        &mut self,
        orchestrator: &dyn Orchestrator,
        node_id: &str,
        verifying_key: VerifyingKey,
    ) -> Result<Task, OrchestratorError> {
        let mut attempts = 0;

        loop {
            // Make the request
            match orchestrator.get_proof_task(node_id, verifying_key).await {
                Ok(task) => {
                    self.request_timer.record_success();
                    return Ok(task);
                }
                Err(e) => {
                    attempts += 1;

                    // Get server-provided retry delay and record failure
                    let server_retry_delay = e
                        .get_retry_after_seconds()
                        .map(|secs| Duration::from_secs(secs as u64))
                        .map(|delay| {
                            delay + crate::consts::cli_consts::rate_limiting::extra_retry_delay()
                        });
                    self.request_timer.record_failure(server_retry_delay);

                    // Check if we should retry
                    if attempts >= self.max_retries || !self.error_handler.should_retry(&e) {
                        return Err(e);
                    }
                }
            }
        }
    }

    /// Submit a proof with automatic retry and server-controlled timing
    pub async fn submit_proof(
        &mut self,
        orchestrator: &dyn Orchestrator,
        task_id: &str,
        proof_hash: &str,
        proof_bytes: Vec<u8>,
        signing_key: SigningKey,
        num_provers: usize,
        task_type: crate::nexus_orchestrator::TaskType,
    ) -> Result<(), OrchestratorError> {
        let mut attempts = 0;

        loop {
            // Make the request
            match orchestrator
                .submit_proof(
                    task_id,
                    proof_hash,
                    proof_bytes.clone(),
                    signing_key.clone(),
                    num_provers,
                    task_type,
                )
                .await
            {
                Ok(()) => {
                    self.request_timer.record_success();
                    return Ok(());
                }
                Err(e) => {
                    attempts += 1;

                    // Get server-provided retry delay and record failure
                    let server_retry_delay = e
                        .get_retry_after_seconds()
                        .map(|secs| Duration::from_secs(secs as u64))
                        .map(|delay| {
                            delay + crate::consts::cli_consts::rate_limiting::extra_retry_delay()
                        });
                    self.request_timer.record_failure(server_retry_delay);

                    // Check if we should retry
                    if attempts >= self.max_retries || !self.error_handler.should_aretry(&e) {
                        return Err(e);
                    }
                }
            }
        }
    }

    /// Get error classification for logging
    pub fn classify_error(&self, error: &OrchestratorError) -> crate::error_classifier::LogLevel {
        self.error_handler.classify_error(error)
    }

    /// Get a mutable reference to the request timer
    pub fn request_timer_mut(&mut self) -> &mut RequestTimer {
        &mut self.request_timer
    }
}
