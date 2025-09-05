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

/// Arguments for creating a new AuthenticatedWorker
pub struct AuthenticatedWorkerArgs {
    pub worker_id: usize,
    pub node_id: u64,
    pub signing_key: SigningKey,
    pub orchestrator: OrchestratorClient,
    pub config: WorkerConfig,
    pub event_sender: mpsc::Sender<Event>,
    pub max_tasks: Option<u32>,
    pub shutdown_sender: broadcast::Sender<()>,
}


/// Single authenticated worker that handles the complete task lifecycle
pub struct AuthenticatedWorker {
    fetcher: TaskFetcher,
    prover: TaskProver,
    submitter: ProofSubmitter,
    event_sender: EventSender,
    max_tasks: Option<u32>,
    tasks_completed: u32,
    shutdown_sender: broadcast::Sender<()>,
    worker_id: usize,
}

impl AuthenticatedWorker {
    pub fn new(
        args: AuthenticatedWorkerArgs,
    ) -> Self {
        let event_sender_helper = EventSender::new(args.event_sender);

        // Create the 3 specialized components
        let fetcher = TaskFetcher::new(
            args.node_id,
            args.signing_key.verifying_key(),
            Box::new(args.orchestrator.clone()),
            event_sender_helper.clone(),
            &args.config,
        );

        let prover = TaskProver::new(event_sender_helper.clone(), args.config.clone(), args.worker_id);

        let submitter = ProofSubmitter::new(
            args.signing_key,
            Box::new(args.orchestrator),
            event_sender_helper.clone(),
            &args.config,
        );

        Self {
            fetcher,
            prover,
            submitter,
            event_sender: event_sender_helper,
            max_tasks: args.max_tasks,
            tasks_completed: 0,
            shutdown_sender: args.shutdown_sender,
            worker_id: args.worker_id,
        }
    }


    /// Start the worker
    pub async fn run(mut self, mut shutdown: broadcast::Receiver<()>) -> Vec<JoinHandle<()>> {
        let mut join_handles = Vec::new();

        // Send initial state
        self.event_sender
            .send_event(Event::state_change(
                ProverState::Waiting,
                format!("Worker {} ready to fetch tasks", self.worker_id),
            ))
            .await;

        // Main work loop
        let worker_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown.recv() => break,
                    should_exit = self.work_cycle() => {
                        if should_exit {
                            break;
                        }
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
    /// Returns true if the worker should exit (max tasks reached)
    async fn work_cycle(&mut self) -> bool {
        // Step 1: Fetch task
        let task = match self.fetcher.fetch_task().await {
            Ok(task) => task,
            Err(_) => {
                // Error already logged in fetcher, wait before retry
                tokio::time::sleep(Duration::from_secs(1)).await;
                return false; // Don't exit on fetch error, just retry
            }
        };

        // Step 2: Prove task
        // Send state change to Proving
        self.event_sender
            .send_event(Event::state_change(
                ProverState::Proving,
                format!("Step 2 of 4: Proving task {}", task.task_id),
            ))
            .await;

        let proof_result = match self.prover.prove_task(&task).await {
            Ok(proof_result) => proof_result,
            Err(_) => {
                // Send state change back to Waiting on proof failure
                self.event_sender
                    .send_event(Event::state_change(
                        ProverState::Waiting,
                        "Proof generation failed, ready for next task".to_string(),
                    ))
                    .await;
                return false; // Don't exit on proof error, just retry
            }
        };

        // Step 3: Submit proof
        let submission_result = self.submitter.submit_proof(&task, &proof_result).await;

        // Only increment task counter on successful submission
        if submission_result.is_ok() {
            self.tasks_completed += 1;

            // Check if we've reached the maximum number of tasks
            if let Some(max) = self.max_tasks {
                if self.tasks_completed >= max {
                    // Give a brief moment for the "Step 4 of 4" message to be processed
                    // before triggering shutdown
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                    self.event_sender
                        .send_event(Event::state_change(
                            ProverState::Waiting,
                            format!("Completed {} tasks, shutting down", self.tasks_completed),
                        ))
                        .await;

                    // Send shutdown signal to trigger application exit
                    let _ = self.shutdown_sender.send(());
                    return true; // Signal to exit the worker loop
                }
            }
        }

        // Send state change back to Waiting at the end of the work cycle
        self.event_sender
            .send_event(Event::state_change(
                ProverState::Waiting,
                "Task completed, ready for next task".to_string(),
            ))
            .await;

        false // Continue with more tasks
    }
}