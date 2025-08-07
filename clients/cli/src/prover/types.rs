//! Proof types and error definitions

use nexus_sdk::stwo::seq::Proof;
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

/// Result of a proof generation, including combined hash for multiple inputs
pub struct ProverResult {
    pub proof: Proof,
    pub combined_hash: String,
}
