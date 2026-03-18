//! Simplified runtime for coordinating authenticated workers

use crate::environment::Environment;
use crate::events::Event;
use crate::orchestrator::OrchestratorClient;
use crate::workers::authenticated_worker::AuthenticatedWorker;
use crate::workers::core::WorkerConfig;
use ed25519_dalek::SigningKey;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

/// Start single authenticated worker
#[allow(clippy::too_many_arguments)]
pub async fn start_authenticated_worker(
    node_id: u64,
    signing_key: SigningKey,
    orchestrator: OrchestratorClient,
    shutdown: broadcast::Receiver<()>,
    environment: Environment,
    client_id: String,
    max_tasks: Option<u32>,
    max_difficulty: Option<crate::nexus_orchestrator::TaskDifficulty>,
    num_workers: usize,
) -> (
    mpsc::Receiver<Event>,
    Vec<JoinHandle<()>>,
    broadcast::Sender<()>,
) {
    let mut config = WorkerConfig::new(environment, client_id);
    config.max_difficulty = max_difficulty;
    config.num_workers = num_workers;
    let (event_sender, event_receiver) =
        mpsc::channel::<Event>(crate::consts::cli_consts::EVENT_QUEUE_SIZE);

    // Create a separate shutdown sender for max tasks completion
    let (shutdown_sender, _) = broadcast::channel(1);

    let worker = AuthenticatedWorker::new(
        node_id,
        signing_key,
        orchestrator,
        config,
        event_sender,
        max_tasks,
        shutdown_sender.clone(),
    );

    let join_handles = worker.run(shutdown).await;
    (event_receiver, join_handles, shutdown_sender)
}
