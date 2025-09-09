//! Proving pipeline that orchestrates the full proving process

use super::engine::ProvingEngine;
use super::input::InputParser;
use super::types::ProverError;
use crate::analytics::track_verification_failed;
use crate::environment::Environment;
use crate::task::Task;
use nexus_sdk::stwo::seq::Proof;
use sha3::{ Digest, Keccak256 };
use tokio::task;
use std::sync::Arc;
use tokio::sync::Semaphore;
/// Orchestrates the complete proving pipeline
pub struct ProvingPipeline;

impl ProvingPipeline {
    /// Execute authenticated proving for a task
    pub async fn prove_authenticated(
        task: &Task,
        environment: &Environment,
        client_id: &str,
        num_workers: usize,
        with_local: bool
    ) -> Result<(Vec<Proof>, String, Vec<String>), ProverError> {
        match task.program_id.as_str() {
            "fib_input_initial" =>
                Self::prove_fib_task(task, environment, client_id, num_workers, with_local).await,
            _ =>
                Err(
                    ProverError::MalformedTask(
                        format!("Unsupported program ID: {}", task.program_id)
                    )
                ),
        }
    }

    // Process fibonacci proving task with multiple inputs
    async fn prove_fib_task_single(
        task: &Task,
        environment: &Environment,
        client_id: &str
    ) -> Result<(Vec<Proof>, String, Vec<String>), ProverError> {
        let all_inputs = task.all_inputs();

        if all_inputs.is_empty() {
            return Err(ProverError::MalformedTask("No inputs provided for task".to_string()));
        }

        let mut proof_hashes = Vec::new();
        let mut all_proofs: Vec<Proof> = Vec::new();

        for (input_index, input_data) in all_inputs.iter().enumerate() {
            // Step 1: Parse and validate input
            let inputs = InputParser::parse_triple_input(input_data)?;
            // Step 2: Generate and verify proof
            let proof = ProvingEngine::prove_and_validate(
                &inputs,
                task,
                environment,
                client_id,
                false
            ).await.map_err(|e| {
                match e {
                    ProverError::Stwo(_) | ProverError::GuestProgram(_) => {
                        // Track verification failure
                        let error_msg = format!("Input {}: {}", input_index as u32 , e);
                        tokio::spawn(
                            track_verification_failed(
                                task.clone(),
                                error_msg.clone(),
                                environment.clone(),
                                client_id.to_string()
                            )
                        );
                        e
                    }
                    _ => e,
                }
            })?;

            // Step 3: Generate proof hash
            let proof_hash = Self::generate_proof_hash(&proof);
            proof_hashes.push(proof_hash);
            all_proofs.push(proof);
        }

        let final_proof_hash = Self::combine_proof_hashes(task, &proof_hashes);

        Ok((all_proofs, final_proof_hash, proof_hashes))
    }

    /// Process fibonacci proving task with multiple inputs
    async fn prove_fib_task(
        task: &Task,
        environment: &Environment,
        client_id: &str,
        num_workers: usize,
        with_local: bool
    ) -> Result<(Vec<Proof>, String, Vec<String>), ProverError> {
        let all_inputs = task.all_inputs();
        if num_workers == 1 || all_inputs.len() <= 1 {
            println!("num_workers: {}, all_inputs.len: {}", num_workers, all_inputs.len());
            return Self::prove_fib_task_single(task, environment, client_id).await;
        }
        if all_inputs.is_empty() {
            return Err(ProverError::MalformedTask("No inputs provided for task".to_string()));
        }
        let semaphore = Arc::new(Semaphore::new(num_workers));
        let mut proof_hashes = Vec::new();
        let mut all_proofs: Vec<Proof> = Vec::new();
        // Create a vector to hold the tasks for concurrent processing
        let mut tasks = vec![];

        for (input_index, input_data) in all_inputs.iter().enumerate() {
            let semaphore = Arc::clone(&semaphore);
            let input_data_clone = input_data.clone(); // Clone for closure
            let task_clone = task.clone();
            let environment_clone = environment.clone();
            let client_id_clone = client_id.to_string();

            // Spawn each task to run concurrently
            let task = task::spawn(async move {
                // 获取一个许可证，控制并发
                let _permit = semaphore.acquire().await.unwrap(); // 阻塞直到获得许可
                // Step 1: Parse and validate input
                let inputs = match InputParser::parse_triple_input(&input_data_clone) {
                    Ok(parsed_inputs) => parsed_inputs,
                    Err(e) => {
                        return Err(e); // Handle parse error
                    }
                };

                // Step 2: Generate and verify proof
                let proof = match
                    ProvingEngine::prove_and_validate(
                        &inputs,
                        &task_clone,
                        &environment_clone,
                        &client_id_clone,
                        with_local
                    ).await
                {
                    Ok(valid_proof) => valid_proof,
                    Err(e) => {
                        // Track verification failure
                        match e {
                            ProverError::Stwo(_) | ProverError::GuestProgram(_) => {
                                let error_msg = format!("Input {}: {}", input_index, e);
                                tokio::spawn(
                                    track_verification_failed(
                                        task_clone.clone(),
                                        error_msg.clone(),
                                        environment_clone.clone(),
                                        client_id_clone.clone()
                                    )
                                );
                            }
                            _ => {}
                        }
                        return Err(e); // Return the error if proof generation fails
                    }
                };

                // Step 3: Generate proof hash
                let proof_hash = Self::generate_proof_hash(&proof);

                Ok((proof_hash, proof)) // Return the generated proof and hash
            });

            // Push the task to the tasks vector
            tasks.push(task);
        }

        // Await all the tasks and collect results
        let results = futures::future::join_all(tasks).await;

        for result in results {
            match result {
                Ok(Ok((proof_hash, proof))) => {
                    proof_hashes.push(proof_hash);
                    all_proofs.push(proof);
                }
                Ok(Err(e)) => {
                    eprintln!("Error processing proof: {}", e);
                }
                Err(e) => {
                    eprintln!("Join error: {}", e);
                }
            }
        }

        let final_proof_hash = Self::combine_proof_hashes(task, &proof_hashes);

        Ok((all_proofs, final_proof_hash, proof_hashes))
    }

    /// Generate hash for a proof
    fn generate_proof_hash(proof: &Proof) -> String {
        let proof_bytes = postcard::to_allocvec(proof).expect("Failed to serialize proof");
        format!("{:x}", Keccak256::digest(&proof_bytes))
    }

    /// Combine multiple proof hashes based on task type
    fn combine_proof_hashes(task: &Task, proof_hashes: &[String]) -> String {
        match task.task_type {
            | crate::nexus_orchestrator::TaskType::AllProofHashes
            | crate::nexus_orchestrator::TaskType::ProofHash => {
                Task::combine_proof_hashes(proof_hashes)
            }
            _ => proof_hashes.first().cloned().unwrap_or_default(),
        }
    }
}
