//! Proof verification

use super::types::ProverError;
use nexus_sdk::{KnownExitCodes, Verifiable, Viewable, stwo::seq::Proof};

/// Proof verifier for validating generated proofs
pub struct ProofVerifier;

impl ProofVerifier {
    /// Verify a proof with expected inputs and exit code
    pub fn verify_proof(
        proof: &Proof,
        inputs: &(u32, u32, u32),
        prover: &nexus_sdk::stwo::seq::Stwo<nexus_sdk::Local>,
    ) -> Result<(), ProverError> {
        match proof.verify_expected::<(u32, u32, u32), ()>(
            inputs,
            KnownExitCodes::ExitSuccess as u32,
            &(),
            &prover.elf,
            &[],
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(ProverError::Stwo(format!(
                "Proof verification failed: {} for inputs: {:?}",
                e, inputs
            ))),
        }
    }

    /// Check exit code from proof execution
    pub fn check_exit_code<T: Viewable>(view: &T) -> Result<(), ProverError> {
        let exit_code = view.exit_code().map_err(|e| {
            ProverError::GuestProgram(format!("Failed to deserialize exit code: {}", e))
        })?;

        if exit_code != KnownExitCodes::ExitSuccess as u32 {
            return Err(ProverError::GuestProgram(format!(
                "Prover exited with non-zero exit code: {}",
                exit_code
            )));
        }

        Ok(())
    }
}
