use crate::analytics::track;
use crate::environment::Environment;
use crate::task::Task;
use log::error;
use nexus_sdk::stwo::seq::Proof;
use nexus_sdk::{KnownExitCodes, Local, Prover, Viewable, stwo::seq::Stwo};
use serde_json::json;
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

    #[error("Analytics tracking error: {0}")]
    Analytics(String),
}

/// Proves a program locally with hardcoded inputs.
pub async fn prove_anonymously(
    environment: &Environment,
    client_id: String,
) -> Result<Proof, ProverError> {
    // The 10th term of the Fibonacci sequence is 55
    let public_input: u32 = 9;

    let stwo_prover = get_default_stwo_prover()?;
    let (view, proof) = stwo_prover
        .prove_with_input::<(), u32>(&(), &public_input)
        .map_err(|e| ProverError::Stwo(format!("Failed to run prover: {}", e)))?;

    let exit_code = view.exit_code().map_err(|e| {
        ProverError::GuestProgram(format!("Failed to deserialize exit code: {}", e))
    })?;

    if exit_code != KnownExitCodes::ExitSuccess as u32 {
        return Err(ProverError::GuestProgram(format!(
            "Prover exited with non-zero exit code: {}",
            exit_code
        )));
    }

    // Send analytics event for anonymous proof
    track(
        "cli_proof_anon_v3".to_string(),
        json!({
            "program_name": "fib_input",
            "public_input": public_input,
        }),
        environment,
        client_id,
    )
    .await
    .map_err(|e| ProverError::Analytics(e.to_string()))?;

    Ok(proof)
}

/// Proves a program with a given node ID
pub async fn authenticated_proving(
    task: &Task,
    environment: &Environment,
    client_id: String,
) -> Result<Proof, ProverError> {
    let public_input = get_public_input(task)?;
    let stwo_prover = get_default_stwo_prover()?;
    let (view, proof) = stwo_prover
        .prove_with_input::<(), u32>(&(), &public_input)
        .map_err(|e| ProverError::Stwo(format!("Failed to run prover: {}", e)))?;

    let exit_code = view.exit_code().map_err(|e| {
        ProverError::GuestProgram(format!("Failed to deserialize exit code: {}", e))
    })?;

    if exit_code != KnownExitCodes::ExitSuccess as u32 {
        return Err(ProverError::GuestProgram(format!(
            "Prover exited with non-zero exit code: {}",
            exit_code
        )));
    }

    // Send analytics event for authenticated proof
    track(
        "cli_proof_node_v3".to_string(),
        json!({
            "program_name": "fib_input",
            "public_input": public_input,
            "task_id": task.task_id,
        }),
        environment,
        client_id,
    )
    .await
    .map_err(|e| ProverError::Analytics(e.to_string()))?;

    Ok(proof)
}

fn get_public_input(task: &Task) -> Result<u32, ProverError> {
    // fib_input expects a single public input as a u32.
    if task.public_inputs.is_empty() {
        return Err(ProverError::MalformedTask(
            "Task public inputs are empty".to_string(),
        ));
    }
    Ok(task.public_inputs[0] as u32)
}

/// Create a Stwo prover for the default program.
pub fn get_default_stwo_prover() -> Result<Stwo<Local>, ProverError> {
    let elf_bytes = include_bytes!("../assets/fib_input");
    Stwo::<Local>::new_from_bytes(elf_bytes).map_err(|e| {
        let msg = format!("Failed to load guest program: {}", e);
        ProverError::Stwo(msg)
    })
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
        let environment = Environment::Local;
        let client_id = "test_client_id".to_string();
        if let Err(e) = prove_anonymously(&environment, client_id).await {
            panic!("Failed to prove anonymously: {}", e);
        }
    }
}
