use std::time::Duration;
use nexus_sdk::{stwo, stwo::seq::Stwo, Local, Prover, Viewable};

use crate::orchestrator_client::OrchestratorClient;
use colored::Colorize;
use log::{error, warn};
use sha3::{Digest, Keccak256};
use thiserror::Error;
use crate::{analytics, config};

#[derive(Error, Debug)]
pub enum ProverError {
    #[error("Orchestrator error: {0}")]
    OrchestratorError(String),

    #[error("Stwo Prover error: {0}")]
    StwoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<postcard::Error> for ProverError {
    fn from(e: postcard::Error) -> Self {
        ProverError::SerializationError(format!("Serialization error: {}", e))
    }
}

/// Starts the prover, which can be anonymous or connected to the Nexus Orchestrator.
/// 
/// # Arguments
/// * `orchestrator_client` - The client to interact with the Nexus Orchestrator.
/// * `node_id` - The ID of the node to connect to. If `None`, the prover will run in anonymous mode.
/// * `_threads` - The number of threads to use for proving.
pub async fn start_prover(
    environment: config::Environment,
    node_id: Option<u64>,
    _threads: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(_node_id) = node_id {
        let _orchestrator_client = OrchestratorClient::new(environment);
        // println!("{}", format!("Node id {}!", node_id).bright_cyan());
        todo!()
    } else {
        println!(
            "\n===== {} =====\n",
            "Starting Anonymous proof generation for programs"
                .bold()
                .underline()
                .bright_cyan()
        );
        run_anonymous_proving_loop(environment).await;
    }

    Ok(())

    // // Run the initial setup to determine anonymous or connected node
    // match setup::run_initial_setup().await {
    //     // If the user selected "anonymous"
    //     setup::SetupResult::Anonymous => {
    //         println!(
    //             "\n===== {} =====\n",
    //             "Starting Anonymous proof generation for programs"
    //                 .bold()
    //                 .underline()
    //                 .bright_cyan()
    //         );
    //         let client_id = format!("{:x}", md5::compute(b"anonymous"));
    //         let mut proof_count = 1;
    //
    //         loop {
    //             println!("\n================================================");
    //             println!(
    //                 "{}",
    //                 format!("\nStarting proof #{} (anonymous) ...\n", proof_count).yellow()
    //             );
    //
    //             // We'll do a few attempts (e.g. 3) in case of transient failures
    //             let max_attempts = 3;
    //             let mut attempt = 1;
    //             let mut success = false;
    //
    //             while attempt <= max_attempts {
    //                 println!("Attempt #{} for anonymous proving", attempt);
    //                 match anonymous_proving() {
    //                     Ok(_) => {
    //                         println!("Anonymous proving succeeded on attempt #{attempt}!");
    //                         success = true;
    //                         break;
    //                     }
    //                     Err(e) => {
    //                         warn!("Attempt #{attempt} failed: {e}");
    //                         attempt += 1;
    //                         if attempt <= max_attempts {
    //                             warn!("Retrying anonymous proving in 2s...");
    //                             tokio::time::sleep(Duration::from_secs(2)).await;
    //                         }
    //                     }
    //                 }
    //             }
    //
    //             if !success {
    //                 error!(
    //                     "All {} attempts to prove anonymously failed. Moving on to next proof iteration.",
    //                     max_attempts
    //                 );
    //             }
    //
    //             proof_count += 1;
    //             analytics::track(
    //                 "cli_proof_anon_v2".to_string(),
    //                 format!("Completed anon proof iteration #{}", proof_count),
    //                 serde_json::json!({
    //                     "node_id": "anonymous",
    //                     "proof_count": proof_count,
    //                 }),
    //                 false,
    //                 environment,
    //                 client_id.clone(),
    //             );
    //
    //             // Sleep before next proof
    //             tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    //         }
    //     }
    //
    //     // If the user selected "connected"
    //     setup::SetupResult::Connected(node_id) => {
    //         println!(
    //             "\n===== {} =====\n",
    //             "Starting proof generation for programs"
    //                 .bold()
    //                 .underline()
    //                 .bright_cyan()
    //         );
    //         let flops = flops::measure_flops()?;
    //         let flops_formatted = format!("{:.2}", flops);
    //         let flops_str = format!("{} FLOPS", flops_formatted);
    //         println!(
    //             "{}: {}",
    //             "Computational capacity of this node".bold(),
    //             flops_str.bright_cyan()
    //         );
    //         println!(
    //             "{}: {}",
    //             "You are proving with node ID".bold(),
    //             node_id.bright_cyan()
    //         );
    //         println!(
    //             "{}: {}",
    //             "Environment".bold(),
    //             environment.to_string().bright_cyan()
    //         );
    //
    //         let client_id = format!("{:x}", md5::compute(node_id.as_bytes()));
    //         let mut proof_count = 1;
    //
    //         loop {
    //             println!("\n================================================");
    //             println!(
    //                 "{}",
    //                 format!(
    //                     "\n[node: {}] Starting proof #{} (connected) ...\n",
    //                     node_id, proof_count
    //                 )
    //                 .yellow()
    //             );
    //
    //             // Retry logic for authenticated_proving
    //             let max_attempts = 3;
    //             let mut attempt = 1;
    //             let mut success = false;
    //
    //             while attempt <= max_attempts {
    //                 println!(
    //                     "Attempt #{} for authenticated proving (node_id={})",
    //                     attempt, node_id
    //                 );
    //                 match authenticated_proving(&node_id, environment).await {
    //                     Ok(_) => {
    //                         println!("Proving succeeded on attempt #{attempt}!");
    //                         success = true;
    //                         break;
    //                     }
    //                     Err(e) => {
    //                         warn!("Attempt #{attempt} failed with error: {e}");
    //                         attempt += 1;
    //                         if attempt <= max_attempts {
    //                             warn!("Retrying in 2s...");
    //                             tokio::time::sleep(Duration::from_secs(2)).await;
    //                         }
    //                     }
    //                 }
    //             }
    //
    //             if !success {
    //                 error!(
    //                     "All {} attempts to prove with node {} failed. Continuing to next proof iteration.",
    //                     max_attempts, node_id
    //                 );
    //             }
    //
    //             proof_count += 1;
    //             analytics::track(
    //                 "cli_proof_node_v2".to_string(),
    //                 format!("Completed proof iteration #{}", proof_count),
    //                 serde_json::json!({
    //                     "node_id": node_id,
    //                     "proof_count": proof_count,
    //                 }),
    //                 false,
    //                 environment,
    //                 client_id.clone(),
    //             );
    //         }
    //     }
    //
    //     // If setup is invalid
    //     setup::SetupResult::Invalid => {
    //         error!("Invalid setup option selected.");
    //         Err("Invalid setup option selected".into())
    //     }
    // }
}

/// Loop indefinitely, creating proofs with hardcoded inputs.
async fn run_anonymous_proving_loop(environment: config::Environment) {
    let client_id = format!("{:x}", md5::compute(b"anonymous"));
    let mut proof_count = 1;
    loop {
        println!("\n================================================");
        println!(
            "{}",
            format!("\nStarting proof #{} (anonymous) ...\n", proof_count).yellow()
        );
        let stwo_prover = get_default_stwo_prover().expect("Failed to create Stwo prover");    
        prove_anonymously(stwo_prover).unwrap();
        proof_count += 1;
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

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

/// Proves a program locally with hardcoded inputs.
fn prove_anonymously(stwo_prover: Stwo<Local>) -> Result<(), ProverError> {
    // The 10th term of the Fibonacci sequence is 55
    let public_input: u32 = 9;
    let proof_bytes = prove_helper(stwo_prover, public_input)?;
    let msg = format!(
        "ZK proof created (anonymous) with size: {} bytes",
        proof_bytes.len()
    );
    println!("{}", msg.green());
    Ok(())
}

/// Proves a program with a given node ID
async fn authenticated_proving(
    node_id: &str,
    orchestrator_client: OrchestratorClient,
    stwo_prover: Stwo<Local>,
) -> Result<(), ProverError> {
    println!("Fetching a task to prove from Nexus Orchestrator...");
    let task = orchestrator_client
        .get_proof_task(node_id)
        .await
        .map_err(|e| {
            ProverError::OrchestratorError(format!("Failed to fetch proof task: {}", e))
        })?;

    let public_input: u32 = task.public_inputs.first().cloned().unwrap_or_default() as u32;
    let proof_bytes = prove_helper(stwo_prover, public_input)?;
    let proof_hash = format!("{:x}", Keccak256::digest(&proof_bytes));
    orchestrator_client
        .submit_proof(&task.task_id, &proof_hash, proof_bytes)
        .await
        .map_err(|e| ProverError::OrchestratorError(format!("Failed to submit proof: {}", e)))?;

    println!("{}", "ZK proof successfully submitted".green());
    Ok(())
}

/// Create a Stwo prover for the default program.
fn get_default_stwo_prover() -> Result<Stwo<Local>, Box<dyn std::error::Error>> {
    let elf_file_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("fib_input");
    Stwo::<Local>::new_from_file(&elf_file_path).map_err(|e| {
        error!("Failed to load guest program: {}", e);
        e.into()
    })
}

fn prove_helper(stwo_prover: Stwo<Local>, public_input: u32) -> Result<Vec<u8>, ProverError> {
    println!("Creating ZK proof with input {}", public_input);
    let (view, proof) = stwo_prover
        .prove_with_input::<(), u32>(&(), &public_input)
        .map_err(|e| ProverError::StwoError(format!("Failed to run prover: {}", e)))?;

    let exit_code = view
        .exit_code()
        .map(|u| u as i32)
        .map_err(|e| ProverError::StwoError(format!("Failed to retrieve exit code: {}", e)))?;
    assert_eq!(exit_code, 0, "Unexpected exit code!");

    postcard::to_allocvec(&proof)
        .map_err(|e| ProverError::SerializationError(format!("Failed to serialize proof: {}", e)))
}