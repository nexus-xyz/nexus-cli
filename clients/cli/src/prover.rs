use crate::orchestrator::{Orchestrator, OrchestratorClient};
use ed25519_dalek::SigningKey;
use nexus_sdk::{Local, Prover, Viewable, stwo::seq::Stwo};
use sha3::{Digest, Keccak256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProverError {
    #[error("Orchestrator error: {0}")]
    Orchestrator(String),

    #[error("Stwo prover error: {0}")]
    Stwo(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] postcard::Error),
}

/// Proves a program locally with hardcoded inputs.
pub fn prove_anonymously() -> Result<(), ProverError> {
    let stwo_prover = get_default_stwo_prover()?;
    // The 10th term of the Fibonacci sequence is 55
    let public_input: u32 = 9;
    let _proof_bytes = prove_helper(stwo_prover, public_input)?;
    Ok(())
}

/// Proves a program with a given node ID
pub async fn authenticated_proving(
    node_id: u64,
    orchestrator_client: &OrchestratorClient,
    stwo_prover: Stwo<Local>,
    signing_key: SigningKey,
) -> Result<(), ProverError> {
    let verifying_key = signing_key.verifying_key();
    let task = orchestrator_client
        .get_proof_task(&node_id.to_string(), verifying_key)
        .await
        .map_err(|e| ProverError::Orchestrator(format!("Failed to fetch proof task: {}", e)))?;

    let public_input: u32 = task.public_inputs.first().cloned().unwrap_or_default() as u32;
    let proof_bytes = prove_helper(stwo_prover, public_input)?;
    let proof_hash = format!("{:x}", Keccak256::digest(&proof_bytes));
    orchestrator_client
        .submit_proof(&task.task_id, &proof_hash, proof_bytes, signing_key)
        .await
        .map_err(|e| ProverError::Orchestrator(format!("Failed to submit proof: {}", e)))?;
    Ok(())
}

/// Create a Stwo prover for the default program.
pub fn get_default_stwo_prover() -> Result<Stwo<Local>, ProverError> {
    let elf_bytes = include_bytes!("../assets/fib_input");
    Stwo::<Local>::new_from_bytes(elf_bytes).map_err(|e| {
        let msg = format!("Failed to load guest program: {}", e);
        ProverError::Stwo(msg)
    })
}

fn prove_helper(stwo_prover: Stwo<Local>, public_input: u32) -> Result<Vec<u8>, ProverError> {
    let (view, proof) = stwo_prover
        .prove_with_input::<(), u32>(&(), &public_input)
        .map_err(|e| ProverError::Stwo(format!("Failed to run prover: {}", e)))?;

    let exit_code = view
        .exit_code()
        .map_err(|e| ProverError::Stwo(format!("Failed to retrieve exit code: {}", e)))?;
    assert_eq!(exit_code, 0, "Unexpected exit code!");

    postcard::to_allocvec(&proof).map_err(ProverError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // The default Stwo prover should be created successfully.
    fn test_get_default_stwo_prover() {
        let prover = get_default_stwo_prover();
        match prover {
            Ok(_) => println!("Prover initialized successfully."),
            Err(e) => panic!("Failed to initialize prover: {}", e),
        }
    }

    #[tokio::test]
    // Proves a program with hardcoded inputs should succeed.
    async fn test_prove_anonymously() {
        let result = prove_anonymously();
        assert!(result.is_ok(), "Anonymous proving failed: {:?}", result);
    }
}
