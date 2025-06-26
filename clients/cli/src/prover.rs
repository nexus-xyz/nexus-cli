use crate::analytics::track;
use crate::environment::Environment;
use crate::task::Task;
use log::{debug, error};
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

    // Send analytics event for anonymous proof - return analytics error but don't fail the proof
    if let Err(e) = track(
        "cli_proof_anon_v3".to_string(),
        json!({
            "program_name": "fib_input_initial",
            "public_input": public_input.0,
            "public_input_2": public_input.1,
            "public_input_3": public_input.2,
        }),
        environment,
        client_id,
    )
    .await
    {
        // Log locally but also return the analytics error so it can be classified and displayed
        debug!("Analytics tracking failed (non-critical): {}", e);
        return Err(ProverError::Analytics(e.to_string()));
    }

    Ok(proof)
}

/// Proves a program with a given node ID
pub async fn authenticated_proving(
    task: &Task,
    environment: &Environment,
    client_id: String,
) -> Result<Proof, ProverError> {
    let (view, proof, analytics_input) = match task.program_id.as_str() {
        "fast-fib" => {
            // fast-fib uses string inputs
            let input = get_string_public_input(task)?;
            let stwo_prover = get_default_stwo_prover()?;
            let (view, proof) = stwo_prover
                .prove_with_input::<(), u32>(&(), &input)
                .map_err(|e| ProverError::Stwo(format!("Failed to run fast-fib prover: {}", e)))?;
            (view, proof, input)
        }
        "fib_input_initial" => {
            let inputs = get_triple_public_input(task)?;
            let stwo_prover = get_initial_stwo_prover()?;
            let (view, proof) = stwo_prover
                .prove_with_input::<(), (u32, u32, u32)>(&(), &inputs)
                .map_err(|e| {
                    ProverError::Stwo(format!("Failed to run fib_input_initial prover: {}", e))
                })?;
            (view, proof, inputs.0)
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

    // Send analytics event for authenticated proof
    let analytics_data = match task.program_id.as_str() {
        "fast-fib" => json!({
            "program_name": "fast-fib",
            "public_input": analytics_input,
            "task_id": task.task_id,
        }),
        "fib_input_initial" => {
            let inputs = get_triple_public_input(task)?;
            json!({
                "program_name": "fib_input_initial",
                "public_input": inputs.0,
                "public_input_2": inputs.1,
                "public_input_3": inputs.2,
                "task_id": task.task_id,
            })
        }
        _ => unreachable!(),
    };

    // Send analytics event for authenticated proof - return analytics error but don't fail the proof
    if let Err(e) = track(
        "cli_proof_node_v3".to_string(),
        analytics_data,
        environment,
        client_id,
    )
    .await
    {
        // Log locally but also return the analytics error so it can be classified and displayed
        debug!("Analytics tracking failed (non-critical): {}", e);
        return Err(ProverError::Analytics(e.to_string()));
    }

    Ok(proof)
}

fn get_string_public_input(task: &Task) -> Result<u32, ProverError> {
    // For fast-fib, just take the first byte as a u32 (how it worked before)
    if task.public_inputs.is_empty() {
        return Err(ProverError::MalformedTask(
            "Task public inputs are empty".to_string(),
        ));
    }
    Ok(task.public_inputs[0] as u32)
}

fn get_triple_public_input(task: &Task) -> Result<(u32, u32, u32), ProverError> {
    if task.public_inputs.len() < 12 {
        return Err(ProverError::MalformedTask(
            "Public inputs buffer too small, expected at least 12 bytes for three u32 values"
                .to_string(),
        ));
    }

    // Read all three u32 values (little-endian) from the buffer
    let mut bytes = [0u8; 4];

    bytes.copy_from_slice(&task.public_inputs[0..4]);
    let n = u32::from_le_bytes(bytes);

    bytes.copy_from_slice(&task.public_inputs[4..8]);
    let init_a = u32::from_le_bytes(bytes);

    bytes.copy_from_slice(&task.public_inputs[8..12]);
    let init_b = u32::from_le_bytes(bytes);

    Ok((n, init_a, init_b))
}

/// Create a Stwo prover for the default program.
pub fn get_default_stwo_prover() -> Result<Stwo<Local>, ProverError> {
    let elf_bytes = include_bytes!("../assets/fib_input");
    Stwo::<Local>::new_from_bytes(elf_bytes).map_err(|e| {
        let msg = format!("Failed to load fib_input guest program: {}", e);
        ProverError::Stwo(msg)
    })
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
