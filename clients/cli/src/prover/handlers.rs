//! High-level proving interface

use super::pipeline::ProvingPipeline;
use super::types::ProverError;
use crate::environment::Environment;
use crate::task::Task;
use nexus_sdk::stwo::seq::Proof;

/// Proves a program with authenticated task inputs
pub async fn authenticated_proving(
    task: &Task,
    environment: &Environment,
    client_id: &str,
) -> Result<(Proof, String), ProverError> {
    ProvingPipeline::prove_authenticated(task, environment, client_id).await
}
