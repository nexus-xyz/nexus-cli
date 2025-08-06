//! Proving pipeline that orchestrates the full proving process

use super::engine::ProvingEngine;
use super::input::InputParser;
use super::types::ProverError;
use crate::analytics::track_verification_failed;
use crate::environment::Environment;
use crate::events::{Event as WorkerEvent, EventType};
use crate::task::Task;
use nexus_sdk::stwo::seq::Proof;
use sha3::{Digest, Keccak256};

/// Orchestrates the complete proving pipeline
pub struct ProvingPipeline;

impl ProvingPipeline {
    /// Execute authenticated proving for a task
    pub async fn prove_authenticated(
        task: &Task,
        environment: &Environment,
        client_id: &str,
        event_sender: Option<&tokio::sync::mpsc::Sender<WorkerEvent>>,
    ) -> Result<(Proof, String), ProverError> {
        match task.program_id.as_str() {
            "fib_input_initial" => {
                Self::prove_fib_task(task, environment, client_id, event_sender).await
            }
            _ => Err(ProverError::MalformedTask(format!(
                "Unsupported program ID: {}",
                task.program_id
            ))),
        }
    }

    /// Process fibonacci proving task with multiple inputs
    async fn prove_fib_task(
        task: &Task,
        environment: &Environment,
        client_id: &str,
        event_sender: Option<&tokio::sync::mpsc::Sender<WorkerEvent>>,
    ) -> Result<(Proof, String), ProverError> {
        let all_inputs = task.all_inputs();

        if all_inputs.is_empty() {
            return Err(ProverError::MalformedTask(
                "No inputs provided for task".to_string(),
            ));
        }

        let mut proof_hashes = Vec::new();
        let mut final_proof = None;

        for (input_index, input_data) in all_inputs.iter().enumerate() {
            Self::send_progress_event(event_sender, task, input_index, all_inputs.len()).await;

            // Step 1: Parse and validate input
            let inputs = InputParser::parse_triple_input(input_data)?;

            // Step 2: Generate and verify proof
            let proof = ProvingEngine::prove_and_validate(&inputs).map_err(|e| {
                // Track verification failure
                let error_msg = format!("Input {}: {}", input_index, e);
                tokio::spawn(track_verification_failed(
                    task.clone(),
                    error_msg.clone(),
                    environment.clone(),
                    client_id.to_string(),
                ));
                e
            })?;

            // Step 3: Generate proof hash
            let proof_hash = Self::generate_proof_hash(&proof);
            proof_hashes.push(proof_hash);
            final_proof = Some(proof);
        }

        let final_proof_hash = Self::combine_proof_hashes(task, &proof_hashes);
        Self::send_completion_event(event_sender, task, all_inputs.len(), &final_proof_hash).await;

        Ok((final_proof.expect("No proof found"), final_proof_hash))
    }

    /// Generate hash for a proof
    fn generate_proof_hash(proof: &Proof) -> String {
        let proof_bytes = postcard::to_allocvec(proof).expect("Failed to serialize proof");
        format!("{:x}", Keccak256::digest(&proof_bytes))
    }

    /// Combine multiple proof hashes based on task type
    fn combine_proof_hashes(task: &Task, proof_hashes: &[String]) -> String {
        if task.task_type == crate::nexus_orchestrator::TaskType::ProofHash {
            Task::combine_proof_hashes(proof_hashes)
        } else {
            proof_hashes.first().cloned().unwrap_or_default()
        }
    }

    /// Send progress event for multi-input tasks
    async fn send_progress_event(
        event_sender: Option<&tokio::sync::mpsc::Sender<WorkerEvent>>,
        task: &Task,
        input_index: usize,
        total_inputs: usize,
    ) {
        if let Some(sender) = event_sender {
            if task.task_type == crate::nexus_orchestrator::TaskType::ProofHash {
                let progress_msg = format!(
                    "Processing input {}/{} for proving task",
                    input_index + 1,
                    total_inputs
                );
                let _ = sender
                    .send(WorkerEvent::prover(0, progress_msg, EventType::Refresh))
                    .await;
            }
        }
    }

    /// Send completion event
    async fn send_completion_event(
        event_sender: Option<&tokio::sync::mpsc::Sender<WorkerEvent>>,
        task: &Task,
        total_inputs: usize,
        final_hash: &str,
    ) {
        if let Some(sender) = event_sender {
            if task.task_type == crate::nexus_orchestrator::TaskType::ProofHash {
                let completion_msg = format!(
                    "Completed proving task with {} input(s), hash: {}...",
                    total_inputs,
                    &final_hash[..8.min(final_hash.len())]
                );
                let _ = sender
                    .send(WorkerEvent::prover(0, completion_msg, EventType::Success))
                    .await;
            }
        }
    }
}
