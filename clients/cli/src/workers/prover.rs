//! Proof generation using existing prover module

use super::core::{EventSender, WorkerConfig};
use crate::error_classifier::LogLevel;
use crate::events::EventType;
use crate::prover::{authenticated_proving, ProverError, ProverResult};
use crate::task::Task;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProveError {
    #[error("Proof generation failed: {0}")]
    Generation(#[from] ProverError),
}

/// Task prover that generates proofs using the existing prover module
pub struct TaskProver {
    event_sender: EventSender,
    config: WorkerConfig,
}

impl TaskProver {
    pub fn new(event_sender: EventSender, config: WorkerConfig) -> Self {
        Self {
            event_sender,
            config,
        }
    }

    /// Generate proof for a task with proper logging
    pub async fn prove_task(&self, task: &Task) -> Result<ProverResult, ProveError> {
        // Log start of proving
        self.event_sender.send_proof_event(
            format!("Step 2 of 4: Proving task {}...", task.task_id),
            EventType::Success,
            LogLevel::Info,
        ).await;

        // Use existing prover module for proof generation
        match authenticated_proving(
            task,
            &self.config.environment,
            &self.config.client_id,
            Some(self.event_sender.sender()), // Pass event sender for progress updates
        ).await {
            Ok((proof, combined_hash)) => {
                // Log successful proof generation
                self.event_sender.send_proof_event(
                    format!("Step 3 of 4: Proof generated for task {}", task.task_id),
                    EventType::Success,
                    LogLevel::Info,
                ).await;

                Ok(ProverResult {
                    proof,
                    combined_hash,
                })
            }
            Err(e) => {
                // Log proof generation failure
                self.event_sender.send_proof_event(
                    format!("Proof generation failed for task {}: {}", task.task_id, e),
                    EventType::Error,
                    LogLevel::Error,
                ).await;

                Err(ProveError::Generation(e))
            }
        }
    }
}