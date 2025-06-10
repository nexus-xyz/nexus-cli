use crate::task::Task;
use log::error;
use nexus_sdk::stwo::seq::Proof;
use nexus_sdk::{Local, Prover, Viewable, stwo::seq::Stwo};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProverError {
    #[error("Stwo prover error: {0}")]
    Stwo(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] postcard::Error),
}

/// Proves a program locally with hardcoded inputs.
#[allow(unused)]
pub fn prove_anonymously() -> Result<(), ProverError> {
    let stwo_prover = get_default_stwo_prover()?;
    // The 10th term of the Fibonacci sequence is 55
    let public_input: u32 = 9;
    let _proof_bytes = prove_helper(stwo_prover, public_input)?;
    Ok(())
}

/// Proves a program with a given node ID
pub async fn authenticated_proving(
    task: Task,
    stwo_prover: Stwo<Local>,
) -> Result<Proof, ProverError> {
    let public_input: u32 = task.public_inputs.first().cloned().unwrap_or_default() as u32;

    let (view, proof) = stwo_prover
        .prove_with_input::<(), u32>(&(), &public_input)
        .map_err(|e| ProverError::Stwo(format!("Failed to run prover: {}", e)))?;

    let exit_code = view
        .exit_code()
        .map_err(|e| ProverError::Stwo(format!("Failed to retrieve exit code: {}", e)))?;
    // TODO: Return an error if the exit code is not 0.
    assert_eq!(exit_code, 0, "Unexpected exit code!");

    Ok(proof)
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
