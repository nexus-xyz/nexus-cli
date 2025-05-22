use nexus_sdk::{stwo::seq::Stwo, Local, Prover, Viewable};
use std::time::Duration;

use crate::orchestrator_client::OrchestratorClient;
use crate::{analytics, environment::Environment};
use colored::Colorize;
use log::{error, info, warn};
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

/// Starts the prover, which can be anonymous or connected to the Nexus Orchestrator.
///
/// # Arguments
/// * `orchestrator_client` - The client to interact with the Nexus Orchestrator.
/// * `node_id` - The ID of the node to connect to. If `None`, the prover will run in anonymous mode.
pub async fn start_prover(
    environment: Environment,
    node_id: Option<u64>,
) -> Result<(), ProverError> {
    match node_id {
        Some(id) => {
            info!("Starting authenticated proving loop for node ID: {}", id);
            run_authenticated_proving_loop(id, environment).await?;
        }
        None => {
            info!("Starting anonymous proving loop");
            run_anonymous_proving_loop(environment).await?;
        }
    }
    Ok(())
}

/// Loop indefinitely, creating proofs with hardcoded inputs.
async fn run_anonymous_proving_loop(environment: Environment) -> Result<(), ProverError> {
    let client_id = format!("{:x}", md5::compute(b"anonymous"));
    let mut proof_count = 1;
    loop {
        info!("{}", "Starting proof (anonymous)".yellow());
        if let Err(e) = prove_anonymously() {
            error!("Failed to create proof: {}", e);
        } else {
            analytics::track(
                "cli_proof_anon_v2".to_string(),
                format!("Completed anon proof iteration #{}", proof_count),
                serde_json::json!({
                    "node_id": "anonymous",
                    "proof_count": proof_count,
                }),
                false,
                &environment,
                client_id.clone(),
            );
        }
        proof_count += 1;
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

/// Proves a program locally with hardcoded inputs.
fn prove_anonymously() -> Result<(), ProverError> {
    let stwo_prover = get_default_stwo_prover()?;
    // The 10th term of the Fibonacci sequence is 55
    let public_input: u32 = 9;
    let proof_bytes = prove_helper(stwo_prover, public_input)?;
    let msg = format!(
        "ZK proof created (anonymous) with size: {} bytes",
        proof_bytes.len()
    );
    info!("{}", msg.green());
    Ok(())
}

/// Loop indefinitely, creating proofs with inputs fetched from the Nexus Orchestrator.
async fn run_authenticated_proving_loop(
    node_id: u64,
    environment: Environment,
) -> Result<(), ProverError> {
    let orchestrator_client = OrchestratorClient::new(environment);
    let mut proof_count = 1;
    loop {
        info!("{}", format!("Starting proof (node: {})", node_id).yellow());

        // Retry logic for authenticated_proving
        const MAX_ATTEMPTS: usize = 3;
        let mut attempt = 1;
        let mut success = false;

        while attempt <= MAX_ATTEMPTS {
            let stwo_prover = get_default_stwo_prover().expect("Failed to create Stwo prover");
            match authenticated_proving(node_id, &orchestrator_client, stwo_prover).await {
                Ok(_) => {
                    info!("Proving succeeded on attempt #{attempt}!");
                    success = true;
                    break;
                }
                Err(e) => {
                    warn!("Attempt #{attempt} failed: {e}");
                    attempt += 1;
                    if attempt <= MAX_ATTEMPTS {
                        warn!("Retrying in 2s...");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        }

        if !success {
            error!(
                "All {} attempts to prove with node {} failed. Continuing to next proof iteration.",
                MAX_ATTEMPTS, node_id
            );
        }

        proof_count += 1;

        let client_id = format!("{:x}", md5::compute(node_id.to_le_bytes()));
        analytics::track(
            "cli_proof_node_v2".to_string(),
            format!("Completed proof iteration #{}", proof_count),
            serde_json::json!({
                "node_id": node_id,
                "proof_count": proof_count,
            }),
            false,
            &environment,
            client_id.clone(),
        );
    }
}

/// Proves a program with a given node ID
async fn authenticated_proving(
    node_id: u64,
    orchestrator_client: &OrchestratorClient,
    stwo_prover: Stwo<Local>,
) -> Result<(), ProverError> {
    info!("Fetching a task to prove from Nexus Orchestrator...");
    let task = orchestrator_client
        .get_proof_task(&node_id.to_string())
        .await
        .map_err(|e| ProverError::Orchestrator(format!("Failed to fetch proof task: {}", e)))?;

    let public_input: u32 = task.public_inputs.first().cloned().unwrap_or_default() as u32;
    let proof_bytes = prove_helper(stwo_prover, public_input)?;
    let proof_hash = format!("{:x}", Keccak256::digest(&proof_bytes));
    orchestrator_client
        .submit_proof(&task.task_id, &proof_hash, proof_bytes)
        .await
        .map_err(|e| ProverError::Orchestrator(format!("Failed to submit proof: {}", e)))?;

    info!("{}", "ZK proof successfully submitted".green());
    Ok(())
}

/// Create a Stwo prover for the default program.
fn get_default_stwo_prover() -> Result<Stwo<Local>, ProverError> {
    let elf_bytes = include_bytes!("../assets/fib_input");
    Stwo::<Local>::new_from_bytes(elf_bytes).map_err(|e| {
        let msg = format!("Failed to load guest program: {}", e);
        error!("{}", msg);
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
