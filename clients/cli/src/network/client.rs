//! Network client with built-in retry and error handling

use super::error_handler::ErrorHandler;
use super::request_timer::RequestTimer;
use crate::consts::cli_consts;
use crate::logging::LogLevel;
use crate::orchestrator::Orchestrator;
use crate::orchestrator::error::OrchestratorError;
use ed25519_dalek::{SigningKey, VerifyingKey};

use std::{cmp::min, time::Duration};

/// Proof submission data grouped by business concern
#[derive(Debug, Clone)]
pub struct ProofSubmission {
    pub task_id: String,
    pub proof_hash: String,
    pub proof_bytes: Vec<u8>,
    pub task_type: crate::nexus_orchestrator::TaskType,
    pub individual_proof_hashes: Vec<String>,
    pub proofs_bytes: Vec<Vec<u8>>, // new: full proofs array
}

impl ProofSubmission {
    pub fn new(
        task_id: String,
        proof_hash: String,
        proof_bytes: Vec<u8>,
        task_type: crate::nexus_orchestrator::TaskType,
    ) -> Self {
        Self {
            task_id,
            proof_hash,
            proof_bytes,
            task_type,
            individual_proof_hashes: Vec::new(),
            proofs_bytes: Vec::new(),
        }
    }

    pub fn with_individual_hashes(mut self, hashes: Vec<String>) -> Self {
        self.individual_proof_hashes = hashes;
        self
    }

    pub fn with_proofs(mut self, proofs: Vec<Vec<u8>>) -> Self {
        self.proofs_bytes = proofs;
        self
    }
}

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
        max_difficulty: crate::nexus_orchestrator::TaskDifficulty,
    ) -> Result<crate::orchestrator::client::ProofTaskResult, OrchestratorError> {
        let mut attempts = 0;

        loop {
            // Make the request
            // Default to Large; callers can adapt or override upstream
            match orchestrator
                .get_proof_task(node_id, verifying_key, max_difficulty)
                .await
            {
                Ok(proof_task_result) => {
                    self.request_timer.record_success();
                    return Ok(proof_task_result);
                }
                Err(e) => {
                    attempts += 1;

                    // Get server-provided retry delay and record failure
                    let server_retry_delay = e
                        .get_retry_after_seconds()
                        .map(|secs| Duration::from_secs(secs as u64))
                        .map(|delay| {
                            min(
                                delay + cli_consts::rate_limiting::extra_retry_delay(),
                                Duration::from_secs(60 * 10),
                            )
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
    /// Returns Ok(attempts) on success or Err((error, attempts)) on failure
    pub async fn submit_proof(
        &mut self,
        orchestrator: &dyn Orchestrator,
        submission: ProofSubmission,
        signing_key: SigningKey,
        num_provers: usize,
    ) -> Result<u32, (OrchestratorError, u32)> {
        let mut attempts = 0;

        loop {
            // Make the request
            match orchestrator
                .submit_proof(
                    &submission.task_id,
                    &submission.proof_hash,
                    submission.proof_bytes.clone(),
                    submission.proofs_bytes.clone(),
                    signing_key.clone(),
                    num_provers,
                    submission.task_type,
                    &submission.individual_proof_hashes,
                )
                .await
            {
                Ok(()) => {
                    attempts += 1;
                    self.request_timer.record_success();
                    return Ok(attempts);
                }
                Err(e) => {
                    attempts += 1;

                    // Get server-provided retry delay and record failure
                    let server_retry_delay = e
                        .get_retry_after_seconds()
                        .map(|secs| Duration::from_secs(secs as u64))
                        .map(|delay| {
                            min(
                                delay + cli_consts::rate_limiting::extra_retry_delay(),
                                Duration::from_secs(60 * 10),
                            )
                        });
                    self.request_timer.record_failure(server_retry_delay);

                    // Check if we should retry
                    if attempts >= self.max_retries || !self.error_handler.should_retry(&e) {
                        return Err((e, attempts));
                    }
                }
            }
        }
    }

    /// Get error classification for logging
    pub fn classify_error(&self, error: &OrchestratorError) -> LogLevel {
        self.error_handler.classify_error(error)
    }

    /// Get a mutable reference to the request timer
    pub fn request_timer_mut(&mut self) -> &mut RequestTimer {
        &mut self.request_timer
    }
}
