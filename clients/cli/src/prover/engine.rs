//! Core proving engine

use crate::prover::verifier;

use super::types::ProverError;
use nexus_sdk::{
    Local, Prover,
    stwo::seq::{Proof, Stwo},
};

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

    /// Generate proof for given inputs using the fibonacci program
    /// Returns the proof and a validation function for the view
    pub fn prove_and_validate(inputs: &(u32, u32, u32)) -> Result<Proof, ProverError> {
        let prover = Self::create_fib_prover()?;
        let (view, proof) = prover
            .prove_with_input::<(), (u32, u32, u32)>(&(), inputs)
            .map_err(|e| {
                ProverError::Stwo(format!(
                    "Failed to generate proof for inputs {:?}: {}",
                    inputs, e
                ))
            })?;

        verifier::ProofVerifier::check_exit_code(&view)?;

        // Verify proof immediately (create fresh prover for verification)
        let verify_prover = Self::create_fib_prover()?;
        verifier::ProofVerifier::verify_proof(&proof, inputs, &verify_prover)?;

        Ok(proof)
    }
}
