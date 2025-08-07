//! Session setup and initialization

use crate::config::Config;
use crate::environment::Environment;
use crate::events::Event;
use crate::orchestrator::OrchestratorClient;
use crate::runtime::start_authenticated_worker;
use ed25519_dalek::SigningKey;
use std::error::Error;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

/// Session data for both TUI and headless modes
#[derive(Debug)]
pub struct SessionData {
    /// Event receiver for worker events
    pub event_receiver: mpsc::Receiver<Event>,
    /// Join handles for worker tasks
    pub join_handles: Vec<JoinHandle<()>>,
    /// Shutdown sender to stop all workers
    pub shutdown_sender: broadcast::Sender<()>,
    /// Shutdown sender for max tasks completion
    pub max_tasks_shutdown_sender: broadcast::Sender<()>,
    /// Node ID
    pub node_id: u64,
    /// Orchestrator client
    pub orchestrator: OrchestratorClient,
    /// Number of workers (for display purposes)
    pub num_workers: usize,
}

/// Sets up an authenticated worker session
///
/// This function handles all the common setup required for both TUI and headless modes:
/// 1. Creates signing key for the prover
/// 2. Sets up shutdown channel
/// 3. Starts authenticated worker
/// 4. Returns session data for mode-specific handling
///
/// # Arguments
/// * `config` - Resolved configuration with node_id and client_id
/// * `env` - Environment to connect to
/// * `max_threads` - Optional maximum number of threads for proving
///
/// # Returns
/// * `Ok(SessionData)` - Successfully set up session
/// * `Err` - Session setup failed
pub async fn setup_session(
    config: Config,
    env: Environment,
    max_threads: Option<u32>,
    max_tasks: Option<u32>,
) -> Result<SessionData, Box<dyn Error>> {
    let node_id = config.node_id.parse::<u64>()?;
    let client_id = config.user_id;

    // Create a signing key for the prover
    let mut csprng = rand_core::OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);

    // Create orchestrator client
    let orchestrator_client = OrchestratorClient::new(env.clone());

    // Clamp the number of workers to [1,8]. Keep this low for now to avoid rate limiting.
    let num_workers: usize = max_threads.unwrap_or(1).clamp(1, 8) as usize;

    // Create shutdown channel - only one shutdown signal needed
    let (shutdown_sender, _) = broadcast::channel(1);

    // Start authenticated worker (only mode we support now)
    let (event_receiver, join_handles, max_tasks_shutdown_sender) = start_authenticated_worker(
        node_id,
        signing_key,
        orchestrator_client.clone(),
        shutdown_sender.subscribe(),
        env,
        client_id,
        max_tasks,
    )
    .await;

    Ok(SessionData {
        event_receiver,
        join_handles,
        shutdown_sender,
        max_tasks_shutdown_sender,
        node_id,
        orchestrator: orchestrator_client,
        num_workers,
    })
}
