use crate::analytics::track_verification_failed;
use crate::environment::Environment;
use crate::events::{Event as WorkerEvent, EventType};
use crate::task::Task;
use log::error;
use nexus_sdk::Verifiable;
use nexus_sdk::stwo::seq::Proof;
use nexus_sdk::{KnownExitCodes, Local, Prover, Viewable, stwo::seq::Stwo};
use sha3::{Digest, Keccak256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProverError {
    #[error("Stwo prover error: {0}")]
    Stwo(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] postcard::Error),

    #[error("Malformed task: {0}")]
    MalformedTask(String),

    #[error("Guest Program error: {0}")]
    GuestProgram(String),
}

/// Proves a program locally with hardcoded inputs.
pub async fn prove_anonymously() -> Result<Proof, ProverError> {
    // Compute the 10th Fibonacci number using fib_input_initial
    // Input: (n=9, init_a=1, init_b=1)
    // This computes F(9) = 55 in the classic Fibonacci sequence starting with 1,1
    // Sequence: F(0)=1, F(1)=1, F(2)=2, F(3)=3, F(4)=5, F(5)=8, F(6)=13, F(7)=21, F(8)=34, F(9)=55
    let public_input: (u32, u32, u32) = (9, 1, 1);

    // Use the new initial ELF file for anonymous proving
    let stwo_prover = get_initial_stwo_prover()?;
    let (view, proof) = stwo_prover
        .prove_with_input::<(), (u32, u32, u32)>(&(), &public_input)
        .map_err(|e| {
            ProverError::Stwo(format!(
                "Failed to run fib_input_initial prover (anonymous): {}",
                e
            ))
        })?;

    let exit_code = view.exit_code().map_err(|e| {
        ProverError::GuestProgram(format!("Failed to deserialize exit code: {}", e))
    })?;

    if exit_code != KnownExitCodes::ExitSuccess as u32 {
        return Err(ProverError::GuestProgram(format!(
            "Prover exited with non-zero exit code: {}",
            exit_code
        )));
    }

    Ok(proof)
}

/// Proves a program with a given node ID
pub async fn authenticated_proving(
    task: &Task,
    environment: &Environment,
    client_id: &str,
    event_sender: Option<&tokio::sync::mpsc::Sender<WorkerEvent>>,
) -> Result<(Proof, String), ProverError> {
    // Check for multiple inputs with proof_required task type (not supported yet)
    // TODO: Uncomment this if we actually receive such tasks from orchestrator
    /*
    if task.all_inputs().len() > 1 {
        if let Some(task_type) = task.task_type {
            if task_type == crate::nexus_orchestrator::TaskType::ProofRequired {
                println!("WARNING: Received task with {} inputs and ProofRequired task type - this is not supported yet", task.all_inputs().len());
                return Err(ProverError::MalformedTask(
                    "Multiple inputs with proof_required task type is not supported yet"
                        .to_string(),
                ));
            }
        }
    }
    */

    let (view, proof, combined_hash) = match task.program_id.as_str() {
        "fib_input_initial" => {
            // Handle multiple inputs if present
            let all_inputs = task.all_inputs();

            // Ensure we have at least one input
            if all_inputs.is_empty() {
                return Err(ProverError::MalformedTask(
                    "No inputs provided for task".to_string(),
                ));
            }

            let mut proof_hashes = Vec::new();
            let mut final_proof = None;
            let mut final_view = None;

            // Process each input set
            for (input_index, input_data) in all_inputs.iter().enumerate() {
                // Send progress event for proof hash tasks
                if let Some(sender) = event_sender {
                    let task_type = task.task_type;
                    if task_type == crate::nexus_orchestrator::TaskType::ProofHash {
                        let progress_msg = format!(
                            "Processing input {}/{} for proving task",
                            input_index + 1,
                            all_inputs.len()
                        );
                        let _ = sender
                            .send(WorkerEvent::prover(
                                0, // Use worker ID 0 for progress events
                                progress_msg,
                                EventType::Refresh,
                            ))
                            .await;
                    }
                }

                let inputs = parse_triple_public_input(input_data)?;
                let stwo_prover = get_initial_stwo_prover()?;
                let elf = stwo_prover.elf.clone();
                let (view, proof) = stwo_prover
                    .prove_with_input::<(), (u32, u32, u32)>(&(), &inputs)
                    .map_err(|e| {
                        ProverError::Stwo(format!(
                            "Failed to run fib_input_initial prover for input {}: {}",
                            input_index, e
                        ))
                    })?;

                // Verify the proof
                match proof.verify_expected::<(u32, u32, u32), ()>(
                    &inputs,
                    nexus_sdk::KnownExitCodes::ExitSuccess as u32,
                    &(),
                    &elf,
                    &[],
                ) {
                    Ok(_) => {
                        // Track analytics for proof validation success (non-blocking)
                    }
                    Err(e) => {
                        let error_msg = format!(
                            "Failed to verify proof for input {}: {} for inputs: {:?}",
                            input_index, e, inputs
                        );
                        // Track analytics for verification failure (non-blocking)
                        tokio::spawn(track_verification_failed(
                            task.clone(),
                            error_msg.clone(),
                            environment.clone(),
                            client_id.to_string(),
                        ));
                        return Err(ProverError::Stwo(error_msg));
                    }
                }

                // Generate proof hash for this input (needed for both task types)
                let proof_bytes = postcard::to_allocvec(&proof).expect("Failed to serialize proof");
                let proof_hash = format!("{:x}", Keccak256::digest(&proof_bytes));
                proof_hashes.push(proof_hash);

                // Store the proof and view for return (we'll use the last one, but the hash will be combined)
                final_proof = Some(proof);
                final_view = Some(view);
            }

            // Always combine proof hashes for ProofHash tasks, even for single inputs
            let task_type = task.task_type;

            let final_proof_hash = if task_type == crate::nexus_orchestrator::TaskType::ProofHash {
                // For ProofHash tasks, always hash the result (even single inputs)
                Task::combine_proof_hashes(&proof_hashes)
            } else {
                // For ProofRequired tasks, return the first proof hash (or empty if no proofs)
                proof_hashes.first().cloned().unwrap_or_default()
            };

            // Send completion event for proof hash tasks
            if let Some(sender) = event_sender {
                if task_type == crate::nexus_orchestrator::TaskType::ProofHash {
                    let completion_msg = format!(
                        "Completed proving task with {} input(s), hash: {}...",
                        all_inputs.len(),
                        &final_proof_hash[..8]
                    );
                    let _ = sender
                        .send(WorkerEvent::prover(
                            0, // Use worker ID 0 for progress events
                            completion_msg,
                            EventType::Success,
                        ))
                        .await;
                }
            }

            // Check if this is a ProofHash task type - if so, discard the proof
            if task_type == crate::nexus_orchestrator::TaskType::ProofHash {
                // For ProofHash tasks, we still return the proof but the submission logic
                // should only use the hash and discard the proof
                (final_view.unwrap(), final_proof.unwrap(), final_proof_hash)
            } else {
                // For ProofRequired tasks, return the actual proof
                (final_view.unwrap(), final_proof.unwrap(), final_proof_hash)
            }
        }
        _ => {
            return Err(ProverError::MalformedTask(format!(
                "Unsupported program ID: {}",
                task.program_id
            )));
        }
    };

    let exit_code = view.exit_code().map_err(|e| {
        ProverError::GuestProgram(format!("Failed to deserialize exit code: {}", e))
    })?;

    if exit_code != KnownExitCodes::ExitSuccess as u32 {
        return Err(ProverError::GuestProgram(format!(
            "Prover exited with non-zero exit code: {}",
            exit_code
        )));
    }

    Ok((proof, combined_hash))
}

fn parse_triple_public_input(input_data: &[u8]) -> Result<(u32, u32, u32), ProverError> {
    if input_data.len() < 12 {
        return Err(ProverError::MalformedTask(
            "Public inputs buffer too small, expected at least 12 bytes for three u32 values"
                .to_string(),
        ));
    }

    // Read all three u32 values (little-endian) from the buffer
    let mut bytes = [0u8; 4];

    bytes.copy_from_slice(&input_data[0..4]);
    let n = u32::from_le_bytes(bytes);

    bytes.copy_from_slice(&input_data[4..8]);
    let init_a = u32::from_le_bytes(bytes);

    bytes.copy_from_slice(&input_data[8..12]);
    let init_b = u32::from_le_bytes(bytes);

    Ok((n, init_a, init_b))
}

/// Create a Stwo prover for the initial program.
pub fn get_initial_stwo_prover() -> Result<Stwo<Local>, ProverError> {
    let elf_bytes = include_bytes!("../assets/fib_input_initial");
    Stwo::<Local>::new_from_bytes(elf_bytes).map_err(|e| {
        let msg = format!("Failed to load fib_input_initial guest program: {}", e);
        ProverError::Stwo(msg)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // The initial Stwo prover should be created successfully.
    fn test_get_initial_stwo_prover() {
        let prover = get_initial_stwo_prover();
        match prover {
            Ok(_) => println!("Prover initialized successfully."),
            Err(e) => panic!("Failed to initialize prover: {}", e),
        }
    }

    #[tokio::test]
    // Proves a program with hardcoded inputs should succeed.
    async fn test_prove_anonymously() {
        match prove_anonymously().await {
            Ok(_) => {
                // Success case - version requirements were met or couldn't be fetched
            }
            Err(e) => {
                panic!("Failed to prove anonymously: {}", e);
            }
        }
    }

    #[tokio::test]
    // Should handle multiple inputs with proof_required task type (now supported).
    async fn test_multiple_inputs_proof_required_success() {
        let mut task = Task::new(
            "test_task".to_string(),
            "fib_input_initial".to_string(),
            // First input: n=2, init_a=1, init_b=1 (computes F(2) = 2)
            vec![2, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            crate::nexus_orchestrator::TaskType::ProofRequired,
        );

        // Add a second input: n=3, init_a=1, init_b=1 (computes F(3) = 3)
        task.public_inputs_list
            .push(vec![3, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0]);

        // Set task type to ProofRequired

        let environment = Environment::Production;
        let client_id = "test_client".to_string();

        match authenticated_proving(&task, &environment, &client_id, None).await {
            Ok((_proof, combined_hash)) => {
                // Should succeed with multiple inputs and return the first proof hash for ProofRequired
                assert!(
                    !combined_hash.is_empty(),
                    "Expected proof hash for ProofRequired task type"
                );
                println!(
                    "Multiple inputs with ProofRequired works (returns first proof hash): {}",
                    combined_hash
                );
            }
            Err(e) => {
                panic!(
                    "Expected success for multiple inputs with ProofRequired: {}",
                    e
                );
            }
        }
    }

    #[tokio::test]
    // Should generate combined hash for multiple inputs with proof_hash task type.
    async fn test_multiple_inputs_combined_hash() {
        let mut task = Task::new(
            "test_task".to_string(),
            "fib_input_initial".to_string(),
            // First input: n=3, init_a=1, init_b=1 (computes F(3) = 3)
            vec![3, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            crate::nexus_orchestrator::TaskType::ProofHash,
        );

        // Add a second input: n=4, init_a=1, init_b=1 (computes F(4) = 5)
        task.public_inputs_list
            .push(vec![4, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0]);

        let environment = Environment::Production;
        let client_id = "test_client".to_string();

        match authenticated_proving(&task, &environment, &client_id, None).await {
            Ok((_proof, combined_hash)) => {
                // Should have a combined hash for multiple inputs
                assert!(
                    !combined_hash.is_empty(),
                    "Expected combined hash for multiple inputs"
                );
                println!("Combined hash: {}", combined_hash);
            }
            Err(e) => {
                panic!("Expected success for multiple inputs: {}", e);
            }
        }
    }

    #[tokio::test]
    // Should return combined hash for single input with ProofHash task type.
    async fn test_single_input_proof_hash_combined_hash() {
        let task = Task::new(
            "test_task".to_string(),
            "fib_input_initial".to_string(),
            // Single input: n=2, init_a=1, init_b=1 (computes F(2) = 2)
            vec![2, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            crate::nexus_orchestrator::TaskType::ProofHash,
        );

        let environment = Environment::Production;
        let client_id = "test_client".to_string();

        match authenticated_proving(&task, &environment, &client_id, None).await {
            Ok((_proof, combined_hash)) => {
                // Should have combined hash for ProofHash task type, even with single input
                assert!(
                    !combined_hash.is_empty(),
                    "Expected combined hash for ProofHash task type"
                );
                println!(
                    "Single input with ProofHash - combined hash: {}",
                    combined_hash
                );
            }
            Err(e) => {
                panic!("Expected success for single input with ProofHash: {}", e);
            }
        }
    }

    #[tokio::test]
    // Should return empty combined hash for single input with ProofRequired task type.
    async fn test_single_input_proof_required_no_combined_hash() {
        let task = Task::new(
            "test_task".to_string(),
            "fib_input_initial".to_string(),
            // Single input: n=2, init_a=1, init_b=1 (computes F(2) = 2)
            vec![2, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            crate::nexus_orchestrator::TaskType::ProofRequired,
        );

        // Set task type to ProofRequired

        let environment = Environment::Production;
        let client_id = "test_client".to_string();

        match authenticated_proving(&task, &environment, &client_id, None).await {
            Ok((_proof, combined_hash)) => {
                // Should have proof hash for single input with ProofRequired
                assert!(
                    !combined_hash.is_empty(),
                    "Expected proof hash for single input with ProofRequired"
                );
                println!(
                    "Single input with ProofRequired - returns proof hash: {}",
                    combined_hash
                );
            }
            Err(e) => {
                panic!(
                    "Expected success for single input with ProofRequired: {}",
                    e
                );
            }
        }
    }

    #[test]
    // Should handle multiple input sets correctly (simple test without zkVM).
    fn test_multiple_input_sets_handling() {
        let mut task = Task::new(
            "test_task".to_string(),
            "fib_input_initial".to_string(),
            // First input: n=2, init_a=1, init_b=1 (computes F(2) = 2)
            vec![2, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            crate::nexus_orchestrator::TaskType::ProofRequired,
        );

        // Add a second input: n=3, init_a=1, init_b=1 (computes F(3) = 3)
        task.public_inputs_list
            .push(vec![3, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0]);

        // Verify we have multiple input sets
        let input_count = task.all_inputs().len();
        assert_eq!(input_count, 2, "Should have 2 input sets");

        // Verify each input set has 12 bytes
        for (i, input) in task.all_inputs().iter().enumerate() {
            assert_eq!(input.len(), 12, "Input {} should have 12 bytes", i);
        }

        println!("Test completed - multiple input sets handled correctly");
    }

    #[test]
    #[should_panic(expected = "fib_input_initial expects exactly 12 bytes, got 8")]
    // Should panic when analytics receives wrong input size.
    fn test_analytics_wrong_input_size() {
        let task = Task::new(
            "test_task".to_string(),
            "fib_input_initial".to_string(),
            // Wrong input size: only 8 bytes instead of 12
            vec![1, 2, 3, 4, 5, 6, 7, 8],
            crate::nexus_orchestrator::TaskType::ProofRequired,
        );

        let environment = Environment::Production;
        let client_id = "test_client".to_string();

        // This should panic with the assertion error
        tokio::runtime::Runtime::new().unwrap().block_on(
            crate::analytics::track_authenticated_proof_analytics(task, environment, client_id),
        );
    }

    #[test]
    // Should work correctly when analytics receives correct input size.
    fn test_analytics_correct_input_size() {
        let task = Task::new(
            "test_task".to_string(),
            "fib_input_initial".to_string(),
            // Correct input size: 12 bytes (3 u32 values)
            vec![2, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            crate::nexus_orchestrator::TaskType::ProofRequired,
        );

        let environment = Environment::Production;
        let client_id = "test_client".to_string();

        // This should not panic
        tokio::runtime::Runtime::new().unwrap().block_on(
            crate::analytics::track_authenticated_proof_analytics(task, environment, client_id),
        );

        println!("Analytics test completed successfully");
    }

    #[tokio::test]
    // Should send progress events for proof hash tasks.
    async fn test_proof_hash_progress_events() {
        let mut task = Task::new(
            "test_task".to_string(),
            "fib_input_initial".to_string(),
            // First input: n=2, init_a=1, init_b=1 (computes F(2) = 2)
            vec![2, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            crate::nexus_orchestrator::TaskType::ProofHash,
        );

        // Add a second input: n=3, init_a=1, init_b=1 (computes F(3) = 3)
        task.public_inputs_list
            .push(vec![3, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0]);

        // Set task type to ProofHash
        task.task_type = crate::nexus_orchestrator::TaskType::ProofHash;

        let environment = Environment::Production;
        let client_id = "test_client".to_string();

        // Create a channel to capture progress events
        let (event_sender, mut event_receiver) = tokio::sync::mpsc::channel::<WorkerEvent>(10);

        match authenticated_proving(&task, &environment, &client_id, Some(&event_sender)).await {
            Ok((_proof, combined_hash)) => {
                // Should have combined hash for multiple inputs with ProofHash
                assert!(
                    !combined_hash.is_empty(),
                    "Expected combined hash for ProofHash task type"
                );

                // Check that progress events were sent
                let mut progress_events = Vec::new();
                while let Ok(event) = event_receiver.try_recv() {
                    progress_events.push(event);
                }

                // Should have at least 2 progress events (one for each input) plus completion
                assert!(
                    progress_events.len() >= 3,
                    "Expected at least 3 progress events, got {}",
                    progress_events.len()
                );

                println!("Progress events sent: {}", progress_events.len());
                for event in progress_events {
                    println!("  - {}", event.msg);
                }
            }
            Err(e) => {
                panic!("Expected success for proof hash task: {}", e);
            }
        }
    }
}
