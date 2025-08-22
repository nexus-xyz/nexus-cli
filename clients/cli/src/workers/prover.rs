//! Proof generation using existing prover module

use super::core::{EventSender, WorkerConfig};
use crate::analytics::track_authenticated_proof_analytics;
use crate::events::EventType;
use crate::logging::LogLevel;
use crate::prover::{ProverError, ProverResult, authenticated_proving};
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
        // Use existing prover module for proof generation
        match authenticated_proving(
            task,
            &self.config.environment,
            &self.config.client_id,
            &self.config.num_workers,
        )
        .await
        {
            Ok((proofs, combined_hash, individual_proof_hashes)) => {
                // Log successful proof generation
                self.event_sender
                    .send_prover_event(
                        0, // Single-threaded prover for now
                        format!("Step 3 of 4: Proof generated for task {}", task.task_id),
                        EventType::Success,
                        LogLevel::Info,
                    )
                    .await;

                tokio::spawn(track_authenticated_proof_analytics(
                    task.clone(),
                    self.config.environment.clone(),
                    self.config.client_id.clone(),
                ));

                Ok(ProverResult {
                    proofs,
                    combined_hash,
                    individual_proof_hashes,
                })
            }
            Err(e) => {
                // Log proof generation failure
                self.event_sender
                    .send_prover_event(
                        0, // Single-threaded prover for now
                        format!("Proof generation failed for task {}: {}", task.task_id, e),
                        EventType::Error,
                        LogLevel::Error,
                    )
                    .await;

                Err(ProveError::Generation(e))
            }
        }
    }
}
