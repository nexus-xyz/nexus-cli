//! Simplified runtime for coordinating authenticated workers

use crate::environment::Environment;
use crate::events::Event;
use crate::orchestrator::OrchestratorClient;
use crate::workers::authenticated_worker::AuthenticatedWorker;
use crate::workers::core::WorkerConfig;
use ed25519_dalek::SigningKey;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

/// Start multiple authenticated workers
#[allow(clippy::too_many_arguments)]
pub async fn start_authenticated_workers(
    node_id: u64,
    signing_key: SigningKey,
    orchestrator: OrchestratorClient,
    shutdown: broadcast::Receiver<()>,
    environment: Environment,
    client_id: String,
    max_tasks: Option<u32>,
    num_workers: usize,
) -> (
    mpsc::Receiver<Event>,
    Vec<JoinHandle<()>>,
    broadcast::Sender<()>,
) {
    let config = WorkerConfig::new(environment, client_id, num_workers);
    let (event_sender, event_receiver) =
        mpsc::channel::<Event>(crate::consts::cli_consts::EVENT_QUEUE_SIZE);

    // Create a separate shutdown sender for max tasks completion
    let (shutdown_sender, _) = broadcast::channel(1);
    let mut all_join_handles = Vec::new();

    for i in 0..num_workers {
        let worker_shutdown = shutdown.resubscribe();
        let worker_event_sender = event_sender.clone();
        let worker_orchestrator = orchestrator.clone();
        let worker_signing_key = signing_key.clone();
        let worker_config = config.clone();
        let worker_shutdown_sender = shutdown_sender.clone();

        let max_tasks_per_worker = if let Some(max) = max_tasks {
            // Distribute tasks among workers, rounding up
            Some((max as f32 / num_workers as f32).ceil() as u32)
        } else {
            None
        };

        let worker_handle = tokio::spawn(async move {
            let worker = AuthenticatedWorker::new(
                i,
                node_id,
                worker_signing_key,
                worker_orchestrator,
                worker_config,
                worker_event_sender,
                max_tasks_per_worker,
                worker_shutdown_sender,
            );
            let handles = worker.run(worker_shutdown).await;
            for handle in handles {
                let _ = handle.await;
            }
        });
        all_join_handles.push(worker_handle);
    }

    (event_receiver, all_join_handles, shutdown_sender)
}