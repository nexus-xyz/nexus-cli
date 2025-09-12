//! Simplified runtime for coordinating authenticated workers

use crate::environment::Environment;
use crate::events::Event;
use crate::orchestrator::OrchestratorClient;
use crate::workers::authenticated_worker::{AuthenticatedWorker, AuthenticatedWorkerArgs};
use crate::workers::core::WorkerConfig;
use ed25519_dalek::SigningKey;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

/// Starts a single authenticated worker that manages multiple prover threads internally.
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

    // Only spawn a single worker, which will then use multiple threads for proving internally.
    // This solves the task backlog issue where multiple workers would fetch tasks concurrently.
    let worker_shutdown = shutdown.resubscribe();
    let worker_shutdown_sender = shutdown_sender.clone(); // Clone for the worker task

    let worker_handle = tokio::spawn(async move {
        let worker_args = AuthenticatedWorkerArgs {
            worker_id: 0, // We only have one worker, so ID is 0
            node_id,
            signing_key,
            orchestrator,
            config,
            event_sender,
            max_tasks, // The single worker gets all the tasks
            shutdown_sender: worker_shutdown_sender,
        };
        let worker = AuthenticatedWorker::new(worker_args);
        let handles = worker.run(worker_shutdown).await;
        for handle in handles {
            let _ = handle.await;
        }
    });
    all_join_handles.push(worker_handle);

    (event_receiver, all_join_handles, shutdown_sender)
}
