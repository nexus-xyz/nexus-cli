use crate::environment::Environment;
use crate::orchestrator::error::OrchestratorError;
use ed25519_dalek::{SigningKey, VerifyingKey};

pub(crate) mod client;
pub use client::OrchestratorClient;
pub mod error;

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait Orchestrator: Send + Sync {
    fn environment(&self) -> &Environment;

    /// Get the user ID associated with a wallet address.
    async fn get_user(&self, wallet_address: &str) -> Result<String, OrchestratorError>;

    /// Registers a new user with the orchestrator.
    async fn register_user(
        &self,
        user_id: &str,
        wallet_address: &str,
    ) -> Result<(), OrchestratorError>;

    /// Registers a new node with the orchestrator.
    async fn register_node(&self, user_id: &str) -> Result<String, OrchestratorError>;

    /// Get the wallet address associated with a node ID.
    async fn get_node(&self, node_id: &str) -> Result<String, OrchestratorError>;

    /// Request a new proof task for the node.
    async fn get_proof_task(
        &self,
        node_id: &str,
        verifying_key: VerifyingKey,
        max_difficulty: crate::nexus_orchestrator::TaskDifficulty,
    ) -> Result<crate::orchestrator::client::ProofTaskResult, OrchestratorError>;

    /// Submits a proof to the orchestrator.
    #[allow(clippy::too_many_arguments)]
    async fn submit_proof(
        &self,
        task_id: &str,
        proof_hash: &str,
        proof: Vec<u8>,
        proofs: Vec<Vec<u8>>,
        signing_key: SigningKey,
        num_provers: usize,
        task_type: crate::nexus_orchestrator::TaskType,
        individual_proof_hashes: &[String],
    ) -> Result<(), OrchestratorError>;
}
