use crate::analytics::track_verification_failed;
use crate::environment::Environment;
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
) -> Result<(Proof, Option<String>), ProverError> {
    // Check for multiple inputs with proof_required task type (not supported yet)
    if task.all_inputs().len() > 1 {
        if let Some(task_type) = task.task_type {
            if task_type == crate::nexus_orchestrator::TaskType::ProofRequired {
                return Err(ProverError::MalformedTask(
                    "Multiple inputs with proof_required task type is not supported yet"
                        .to_string(),
                ));
            }
        }
    }

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
                
                // Generate proof hash for this input
                let proof_bytes = postcard::to_allocvec(&proof).expect("Failed to serialize proof");
                let proof_hash = format!("{:x}", Keccak256::digest(&proof_bytes));
                proof_hashes.push(proof_hash);
                
                // Store the proof and view for return (we'll use the last one, but the hash will be combined)
                final_proof = Some(proof);
                final_view = Some(view);
            }
            
            // If we have multiple inputs, combine the proof hashes
            let final_proof_hash = if proof_hashes.len() > 1 {
                Some(Task::combine_proof_hashes(&proof_hashes))
            } else {
                None
            };
            
            // Check if this is a ProofHash task type - if so, discard the proof
            let task_type = task.task_type.unwrap_or(crate::nexus_orchestrator::TaskType::ProofRequired);
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
    // Should return error for multiple inputs with proof_required task type.
    async fn test_multiple_inputs_proof_required_error() {
        let mut task = Task::new(
            "test_task".to_string(),
            "fib_input_initial".to_string(),
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        );
        
        // Add a second input
        task.public_inputs_list
            .push(vec![13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24]);
        
        // Set task type to ProofRequired
        task.task_type = Some(crate::nexus_orchestrator::TaskType::ProofRequired);
        
        let environment = Environment::Production;
        let client_id = "test_client".to_string();
        
        match authenticated_proving(&task, &environment, &client_id).await {
            Ok(_) => panic!("Expected error for multiple inputs with proof_required task type"),
            Err(e) => {
                assert!(e.to_string().contains(
                    "Multiple inputs with proof_required task type is not supported yet"
                ));
            }
        }
    }

    #[tokio::test]
    // Should generate combined hash for multiple inputs with proof_hash task type.
    async fn test_multiple_inputs_combined_hash() {
        let mut task = Task::new(
            "test_task".to_string(),
            "fib_input_initial".to_string(),
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        );
        
        // Add a second input
        task.public_inputs_list
            .push(vec![13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24]);
        
        // Set task type to ProofHash (or None, which should work)
        task.task_type = None;
        
        let environment = Environment::Production;
        let client_id = "test_client".to_string();
        
        match authenticated_proving(&task, &environment, &client_id).await {
            Ok((_proof, combined_hash)) => {
                // Should have a combined hash for multiple inputs
                assert!(combined_hash.is_some(), "Expected combined hash for multiple inputs");
                println!("Combined hash: {}", combined_hash.unwrap());
            }
            Err(e) => {
                panic!("Expected success for multiple inputs: {}", e);
            }
        }
    }
}
