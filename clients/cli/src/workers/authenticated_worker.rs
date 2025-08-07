//! Single authenticated worker that orchestrates fetch→prove→submit

use super::core::{EventSender, WorkerConfig};
use super::fetcher::TaskFetcher;
use super::prover::TaskProver;
use super::submitter::ProofSubmitter;
use crate::events::{Event, ProverState};
use crate::orchestrator::OrchestratorClient;

use ed25519_dalek::SigningKey;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

/// Single authenticated worker that handles the complete task lifecycle
pub struct AuthenticatedWorker {
    fetcher: TaskFetcher,
    prover: TaskProver,
    submitter: ProofSubmitter,
    event_sender: EventSender,
}

impl AuthenticatedWorker {
    pub fn new(
        node_id: u64,
        signing_key: SigningKey,
        orchestrator: OrchestratorClient,
        config: WorkerConfig,
        event_sender: mpsc::Sender<Event>,
    ) -> Self {
        let event_sender_helper = EventSender::new(event_sender);

        // Create the 3 specialized components
        let fetcher = TaskFetcher::new(
            node_id,
            signing_key.verifying_key(),
            Box::new(orchestrator.clone()),
            event_sender_helper.clone(),
            &config,
        );

        let prover = TaskProver::new(event_sender_helper.clone(), config.clone());

        let submitter = ProofSubmitter::new(
            signing_key,
            Box::new(orchestrator),
            event_sender_helper.clone(),
            &config,
        );

        Self {
            fetcher,
            prover,
            submitter,
            event_sender: event_sender_helper,
        }
    }

    /// Start the worker
    pub async fn run(mut self, mut shutdown: broadcast::Receiver<()>) -> Vec<JoinHandle<()>> {
        let mut join_handles = Vec::new();

        // Send initial state
        self.event_sender
            .send_event(Event::state_change(
                ProverState::Waiting,
                "Ready to fetch tasks".to_string(),
            ))
            .await;

        // Main work loop
        let worker_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown.recv() => break,
                    _ = self.work_cycle() => {
                        // Natural rate limiting through work cycle
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        });
        join_handles.push(worker_handle);

        join_handles
    }

    /// Complete work cycle: fetch→prove→submit
    async fn work_cycle(&mut self) {
        // Step 1: Fetch task
        let task = match self.fetcher.fetch_task().await {
            Ok(task) => task,
            Err(_) => {
                // Error already logged in fetcher, wait before retry
                tokio::time::sleep(Duration::from_secs(1)).await;
                return;
            }
        };

        // Step 2: Prove task
        // Send state change to Proving
        self.event_sender
            .send_event(Event::state_change(
                ProverState::Proving,
                format!("Proving task {}", task.task_id),
            ))
            .await;

        let proof_result = match self.prover.prove_task(&task).await {
            Ok(proof_result) => {
                // Send state change back to Waiting after successful proof
                self.event_sender
                    .send_event(Event::state_change(
                        ProverState::Waiting,
                        "Proof completed, ready for next task".to_string(),
                    ))
                    .await;
                proof_result
            }
            Err(_) => {
                // Send state change back to Waiting after failed proof
                self.event_sender
                    .send_event(Event::state_change(
                        ProverState::Waiting,
                        "Proof failed, ready for next task".to_string(),
                    ))
                    .await;
                // Error already logged in prover, continue to next task
                return;
            }
        };

        // Step 3: Submit proof
        let _ = self.submitter.submit_proof(&task, &proof_result).await;
        // Result doesn't matter - error already logged in submitter
        // Continue to next work cycle regardless of submission result
    }
}
