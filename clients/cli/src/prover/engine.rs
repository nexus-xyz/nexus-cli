//! Core proving engine

use crate::prover::verifier;

use super::types::ProverError;
use nexus_sdk::{
    Local, Prover,
    stwo::seq::{Proof, Stwo},
};
use tokio::process::Command;
use std::process::Stdio;
use std::env;
use postcard::{from_bytes, to_allocvec};
use serde_json;

/// Core proving engine for ZK proof generation
    pub struct ProvingEngine;

    impl ProvingEngine {
    /// Create a Stwo prover instance for the fibonacci program
    pub fn create_fib_prover() -> Result<Stwo<Local>, ProverError> {
        let elf_bytes = include_bytes!("../../assets/fib_input_initial");
        Stwo::<Local>::new_from_bytes(elf_bytes).map_err(|e| {
            ProverError::Stwo(format!(
                "Failed to load fib_input_initial guest program: {}",
                e
            ))
        })
    }

    /// Subprocess entrypoint: generate proof without verification
    pub fn prove_fib_subprocess(inputs: &(u32, u32, u32)) -> Result<Proof, ProverError> {
        let prover = Self::create_fib_prover()?;
        let (view, proof) = prover
            .prove_with_input::<(), (u32, u32, u32)>(&(), inputs)
            .map_err(|e| {
                ProverError::Stwo(format!(
                    "Failed to generate proof for inputs {:?}: {}",
                    inputs, e
                ))
            })?;
        // Check exit code in subprocess
        verifier::ProofVerifier::check_exit_code(&view)?;

        Ok(proof)
    }

    /// Generate proof for given inputs using the fibonacci program in a subprocess
    pub async fn prove_and_validate(inputs: &(u32, u32, u32)) -> Result<Proof, ProverError> {
        // Spawn a subprocess for proof generation to isolate memory usage
        let exe_path = env::current_exe()?;
        let mut cmd = Command::new(exe_path);
        cmd.arg("prove-fib-subprocess")
            .arg("--inputs")
            .arg(serde_json::to_string(inputs)?)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let output = cmd.output().await?;
        if !output.status.success() {
            return Err(ProverError::Subprocess(format!(
                "Prover subprocess failed with status: {}",
                output.status
            )));
        }

        // Deserialize proof from subprocess stdout
        let proof: Proof = from_bytes(&output.stdout)?;

        // Verify proof in main process
        let verify_prover = Self::create_fib_prover()?;
        verifier::ProofVerifier::verify_proof(&proof, inputs, &verify_prover)?;

        Ok(proof)
    }
}
