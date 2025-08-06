//! Proof submission with network retry logic

use super::core::{EventSender, WorkerConfig};
use crate::analytics::{track_proof_accepted, track_proof_submission_success};
use crate::consts::cli_consts::{proof_submission, rate_limiting};
use crate::error_classifier::LogLevel;
use crate::events::EventType;
use crate::network::{NetworkClient, RequestTimer, RequestTimerConfig};
use crate::orchestrator::Orchestrator;
use crate::prover::ProverResult;
use crate::task::Task;
use ed25519_dalek::SigningKey;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SubmitError {
    #[error("Network error: {0}")]
    Network(#[from] crate::orchestrator::error::OrchestratorError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] postcard::Error),
}

/// Proof submitter with built-in retry and error handling
pub struct ProofSubmitter {
    signing_key: SigningKey,
    orchestrator: Box<dyn Orchestrator>,
    network_client: NetworkClient,
    event_sender: EventSender,
    config: WorkerConfig,
}

impl ProofSubmitter {
    pub fn new(
        signing_key: SigningKey,
        orchestrator: Box<dyn Orchestrator>,
        event_sender: EventSender,
        config: WorkerConfig,
    ) -> Self {
        // Configure request timer for proof submission
        let timer_config = RequestTimerConfig::combined(
            proof_submission::rate_limit_interval(),
            rate_limiting::SUBMISSION_MAX_REQUESTS_PER_WINDOW,
            rate_limiting::submission_window(),
            proof_submission::initial_backoff(), // Use as default retry delay
        );
        let request_timer = RequestTimer::new(timer_config);

        // Create network client with more retries for critical submissions
        let network_client = NetworkClient::new(request_timer, proof_submission::MAX_RETRIES);

        Self {
            signing_key,
            orchestrator,
            network_client,
            event_sender,
            config,
        }
    }

    /// Submit proof with automatic retry and proper logging
    pub async fn submit_proof(&mut self, task: &Task, proof_result: &ProverResult) -> Result<(), SubmitError> {
        // Log start of submission
        self.event_sender.send_proof_event(
            format!("Step 3 of 4: Submitting proof for task {}...", task.task_id),
            EventType::Success,
            LogLevel::Info,
        ).await;

        // Serialize proof
        let proof_bytes = postcard::to_allocvec(&proof_result.proof)?;

        // Submit through network client with retry logic
        match self.network_client.submit_proof(
            self.orchestrator.as_ref(),
            &task.task_id,
            &proof_result.combined_hash,
            proof_bytes,
            self.signing_key.clone(),
            1, // num_provers (single worker)
            task.task_type,
        ).await {
            Ok(()) => {
                // Log successful submission
                self.event_sender.send_proof_event(
                    format!("Step 4 of 4: Proof submitted successfully for task {}", task.task_id),
                    EventType::Success,
                    LogLevel::Info,
                ).await;

                // Track analytics for successful submission
                self.track_successful_submission(task).await;

                Ok(())
            }
            Err(e) => {
                // Log submission failure with appropriate level
                let log_level = self.network_client.classify_error(&e);
                self.event_sender.send_proof_event(
                    format!("Failed to submit proof for task {}: {}", task.task_id, e),
                    EventType::Error,
                    log_level,
                ).await;

                Err(SubmitError::Network(e))
            }
        }
    }

    /// Track successful submission analytics based on task type
    async fn track_successful_submission(&self, task: &Task) {
        if task.task_type == crate::nexus_orchestrator::TaskType::ProofHash {
            tokio::spawn(track_proof_accepted(
                task.clone(),
                self.config.environment.clone(),
                self.config.client_id.clone(),
            ));
        } else {
            tokio::spawn(track_proof_submission_success(
                task.clone(),
                self.config.environment.clone(),
                self.config.client_id.clone(),
            ));
        }
    }
}