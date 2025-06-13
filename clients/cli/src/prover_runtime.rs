//! Prover Runtime
//!
//! Handles background execution of proof tasks in both authenticated and anonymous modes.
//! Spawns async workers, dispatches tasks, and reports progress back to the UI.
//!
//! Includes:
//! - Task fetching from the orchestrator (authenticated mode)
//! - Worker management and task dispatching
//! - Prover event reporting

use crate::orchestrator::error::OrchestratorError;
use crate::orchestrator::{Orchestrator, OrchestratorClient};
use crate::prover::authenticated_proving;
use crate::task::Task;
use chrono::Local;
use ed25519_dalek::{SigningKey, VerifyingKey};
use nexus_sdk::stwo::seq::Proof;
use sha3::{Digest, Keccak256};
use std::fmt::Display;
use std::string::ToString;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Worker {
    /// Worker that fetches tasks from the orchestrator and processes them.
    TaskFetcher,
    /// Worker that performs proving tasks.
    Prover(usize),
    /// Worker that submits proofs to the orchestrator.
    ProofSubmitter,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display)]
pub enum EventType {
    Success,
    Error,
    Refresh,
    Shutdown,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Event {
    pub worker: Worker,
    pub msg: String,
    pub timestamp: String,
    pub event_type: EventType,
}
impl Event {
    pub fn new(kind: Worker, msg: String, event_type: EventType) -> Self {
        Self {
            worker: kind,
            msg,
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            event_type,
        }
    }

    pub fn task_fetcher(msg: String, event_type: EventType) -> Self {
        Self::new(Worker::TaskFetcher, msg, event_type)
    }

    pub fn prover(worker_id: usize, msg: String, event_type: EventType) -> Self {
        Self::new(Worker::Prover(worker_id), msg, event_type)
    }

    pub fn proof_submitter(msg: String, event_type: EventType) -> Self {
        Self::new(Worker::ProofSubmitter, msg, event_type)
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let worker_type: String = match self.worker {
            Worker::TaskFetcher => "Task Fetcher".to_string(),
            Worker::Prover(worker_id) => format!("Prover {}", worker_id),
            Worker::ProofSubmitter => "Proof Submitter".to_string(),
        };
        write!(
            f,
            "{} [{}] {}: {}",
            self.event_type, self.timestamp, worker_type, self.msg
        )
    }
}

// Queue sizes. Chosen to be larger than the tasks API page size (currently, 50)
const TASK_QUEUE_SIZE: usize = 100;
const EVENT_QUEUE_SIZE: usize = 100;
const RESULT_QUEUE_SIZE: usize = 100;

/// Starts authenticated workers that fetch tasks from the orchestrator and process them.
pub async fn start_authenticated_workers(
    node_id: u64,
    signing_key: SigningKey,
    orchestrator: OrchestratorClient,
    num_workers: usize,
    shutdown: broadcast::Receiver<()>,
) -> (mpsc::Receiver<Event>, Vec<JoinHandle<()>>) {
    let mut join_handles = Vec::new();
    // Worker events
    let (event_sender, event_receiver) = mpsc::channel::<Event>(EVENT_QUEUE_SIZE);

    // Task fetching
    let (task_sender, task_receiver) = mpsc::channel::<Task>(TASK_QUEUE_SIZE);
    let verifying_key = signing_key.verifying_key();
    let fetch_prover_tasks_handle = {
        let orchestrator = orchestrator.clone();
        let event_sender = event_sender.clone();
        let shutdown = shutdown.resubscribe(); // Clone the receiver for task fetching
        tokio::spawn(async move {
            fetch_prover_tasks(
                node_id,
                verifying_key,
                Box::new(orchestrator),
                task_sender,
                event_sender,
                shutdown,
            )
            .await;
        })
    };
    join_handles.push(fetch_prover_tasks_handle);

    // Workers
    let (result_sender, result_receiver) = mpsc::channel::<(Task, Proof)>(RESULT_QUEUE_SIZE);

    let (worker_senders, worker_handles) = start_workers(
        num_workers,
        result_sender.clone(),
        event_sender.clone(),
        shutdown.resubscribe(),
    );
    join_handles.extend(worker_handles);

    // Dispatch tasks to workers
    let dispatcher_handle = start_dispatcher(task_receiver, worker_senders, shutdown.resubscribe());
    join_handles.push(dispatcher_handle);

    // Send proofs to the orchestrator
    let submit_proofs_handle = submit_proofs(
        signing_key,
        Box::new(orchestrator),
        result_receiver,
        event_sender.clone(),
        shutdown.resubscribe(),
    )
    .await;
    join_handles.push(submit_proofs_handle);

    (event_receiver, join_handles)
}

/// Starts anonymous workers that repeatedly prove a program with hardcoded inputs.
pub async fn start_anonymous_workers(
    num_workers: usize,
    shutdown: broadcast::Receiver<()>,
) -> (mpsc::Receiver<Event>, Vec<JoinHandle<()>>) {
    let (event_sender, event_receiver) = mpsc::channel::<Event>(100);
    let mut join_handles = Vec::new();
    for worker_id in 0..num_workers {
        let prover_event_sender = event_sender.clone();
        let mut shutdown_rx = shutdown.resubscribe(); // clone receiver for each worker

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        let message = format!("Worker {} received shutdown signal", worker_id);
                        let _ = prover_event_sender
                            .send(Event::prover(worker_id, message, EventType::Shutdown))
                            .await;
                        break; // Exit the loop on shutdown signal
                    }

                    _ = tokio::time::sleep(Duration::from_millis(300)) => {
                        // Perform work
                        match crate::prover::prove_anonymously() {
                            Ok(_proof) => {
                                let message = "Anonymous proof completed successfully".to_string();
                                let _ = prover_event_sender
                                    .send(Event::prover(worker_id, message, EventType::Success)).await;
                            }
                            Err(e) => {
                                let message = format!("Anonymous Worker: Error - {}", e);
                                let _ =  prover_event_sender.send(Event::prover(worker_id, message, EventType::Error)).await;
                            }
                        }
                    }
                }
            }
        });
        join_handles.push(handle);
    }

    (event_receiver, join_handles)
}

/// Fetches tasks from the orchestrator and place them in the task queue.
pub async fn fetch_prover_tasks(
    node_id: u64,
    verifying_key: VerifyingKey,
    orchestrator_client: Box<dyn Orchestrator>,
    sender: mpsc::Sender<Task>,
    event_sender: mpsc::Sender<Event>,
    mut shutdown: broadcast::Receiver<()>,
) {
    let mut fetch_existing_tasks = true;
    loop {
        tokio::select! {
            _ = shutdown.recv() => {
                break;
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // Get existing tasks.
                if fetch_existing_tasks {
                    match orchestrator_client.get_tasks(&node_id.to_string()).await {
                        Ok(tasks) => {
                            // 🔄
                            let msg = format!("Fetched {} tasks", tasks.len());
                            let _ = event_sender
                                        .send(Event::task_fetcher(msg, EventType::Refresh))
                                        .await;
                            for task in tasks {
                                if sender.send(task).await.is_err() {
                                    let _ = event_sender
                                        .send(Event::task_fetcher("Task queue is closed".to_string(), EventType::Shutdown))
                                        .await;
                                }
                            }
                            fetch_existing_tasks = false;
                        }
                        Err(e) => {
                            // ⚠️
                            let msg = format!("Failed to fetch existing tasks: {}", e);
                            let _ = event_sender
                                .send(Event::task_fetcher(msg, EventType::Error))
                                .await;
                        }
                    }
                }
                match orchestrator_client
                    .get_proof_task(&node_id.to_string(), verifying_key)
                    .await
                {
                    Ok(task) => {
                        if sender.send(task).await.is_err() {
                            let _ = event_sender
                                .send(Event::task_fetcher("Task queue is closed".to_string(), EventType::Shutdown))
                                .await;
                        }
                    }
                    Err(e) => {
                        if let OrchestratorError::Http { status, message: _ } = e {
                            if status == 429 {
                                fetch_existing_tasks = true;
                            }
                        }
                    }
                }

            }
        }
    }
}

/// Submits proofs to the orchestrator
pub async fn submit_proofs(
    signing_key: SigningKey,
    orchestrator: Box<dyn Orchestrator>,
    mut results: mpsc::Receiver<(Task, Proof)>,
    event_sender: mpsc::Sender<Event>,
    mut shutdown: broadcast::Receiver<()>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                maybe_item = results.recv() => {
                    match maybe_item {
                        Some((task, proof)) => {
                            let proof_bytes = postcard::to_allocvec(&proof)
                                .expect("Failed to serialize proof");
                            let proof_hash = format!("{:x}", Keccak256::digest(&proof_bytes));
                            // let now = Local::now();
                            // let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
                            match orchestrator
                                .submit_proof(&task.task_id, &proof_hash, proof_bytes, signing_key.clone())
                                .await
                            {
                                Ok(_) => {
                                    // ✅
                                    let msg = format!(
                                        "Successfully submitted proof for task {}",
                                        task.task_id
                                    );
                                    let _ = event_sender
                                        .send(Event::proof_submitter(msg, EventType::Success))
                                        .await;
                            }
                                Err(OrchestratorError::Http {status, message: _message}) => {
                                    let msg = format!(
                                        "Failed to submit proof for task {}. Status: {}",
                                        task.task_id,
                                        status,
                                    );
                                    let _ = event_sender
                                        .send(Event::proof_submitter(msg, EventType::Error))
                                        .await;
                                }
                                Err(e) => {
                                    let msg = format!(
                                        "Failed to submit proof for task {}: {}",
                                        task.task_id,
                                        e
                                    );
                                    let _ = event_sender
                                        .send(Event::proof_submitter(msg, EventType::Error))
                                        .await;
                                }
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }

                _ = shutdown.recv() => {
                    break;
                }
            }
        }
    })
}

/// Spawns a dispatcher that forwards tasks to available workers in round-robin fashion.
pub fn start_dispatcher(
    mut task_receiver: mpsc::Receiver<Task>,
    worker_senders: Vec<mpsc::Sender<Task>>,
    mut shutdown: broadcast::Receiver<()>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut next_worker = 0;
        loop {
            tokio::select! {
                Some(task) = task_receiver.recv() => {
                    let target = next_worker % worker_senders.len();
                    if let Err(_e) = worker_senders[target].send(task).await {
                        // Channel is closed, stop dispatching tasks
                        return;
                    }
                    next_worker += 1;
                }

                _ = shutdown.recv() => {
                    break;
                }
            }
        }
    })
}

/// Spawns a set of worker tasks that receive tasks and send prover events.
///
/// # Arguments
/// * `num_workers` - The number of worker tasks to spawn.
/// * `results_sender` - The channel to emit results (task and proof).
/// * `prover_event_sender` - The channel to send prover events to the main thread.
///
/// # Returns
/// A tuple containing:
/// * A vector of `Sender<Task>` for each worker, allowing tasks to be sent to them.
/// * A vector of `JoinHandle<()>` for each worker, allowing the main thread to await their completion.
pub fn start_workers(
    num_workers: usize,
    results_sender: mpsc::Sender<(Task, Proof)>,
    event_sender: mpsc::Sender<Event>,
    shutdown: broadcast::Receiver<()>,
) -> (Vec<mpsc::Sender<Task>>, Vec<JoinHandle<()>>) {
    let mut senders = Vec::with_capacity(num_workers);
    let mut handles = Vec::with_capacity(num_workers);

    for worker_id in 0..num_workers {
        let (task_sender, mut task_receiver) = mpsc::channel::<Task>(8);
        // Clone senders and receivers for each worker.
        let prover_event_sender = event_sender.clone();
        let results_sender = results_sender.clone();
        let mut shutdown = shutdown.resubscribe();
        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown.recv() => {
                        let message = format!("Worker {} received shutdown signal", worker_id);
                        let _ = prover_event_sender
                            .send(Event::prover(worker_id, message, EventType::Shutdown))
                            .await;
                        break; // Exit the loop on shutdown signal
                    }
                    // Check if there are tasks to process
                    Some(task) = task_receiver.recv() => {
                        match authenticated_proving(&task).await {
                            Ok(proof) => {
                                let message = format!(
                                    "Proof completed successfully (Prover {})",
                                    worker_id
                                );
                                let _ = prover_event_sender
                                    .send(Event::prover(worker_id, message, EventType::Success))
                                    .await;
                                let _ = results_sender.send((task, proof)).await;
                            }
                            Err(e) => {
                                let message = format!("Error: {}", e);
                                let _ = prover_event_sender
                                    .send(Event::prover(worker_id, message, EventType::Error))
                                    .await;
                            }
                        }
                    }
                    else => break,
                }
            }
        });

        senders.push(task_sender);
        handles.push(handle);
    }

    (senders, handles)
}

#[cfg(test)]
mod tests {
    use crate::orchestrator::MockOrchestrator;
    use crate::prover_runtime::{Event, fetch_prover_tasks};
    use crate::task::Task;
    use std::time::Duration;
    use tokio::sync::{broadcast, mpsc};

    /// Creates a mock orchestrator client that simulates fetching tasks.
    fn get_mock_orchestrator_client() -> MockOrchestrator {
        let mut i = 0;
        let mut mock = MockOrchestrator::new();
        mock.expect_get_proof_task().returning_st(move |_, _| {
            // Simulate a task with dummy data
            let task = Task::new(i.to_string(), format!("Task {}", i), vec![1, 2, 3]);
            i += 1;
            Ok(task)
        });
        mock
    }

    #[tokio::test]
    // Should fetch and enqueue tasks from the orchestrator.
    async fn test_task_fetching() {
        let orchestrator_client = Box::new(get_mock_orchestrator_client());
        let signer_key = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        let verifying_key = signer_key.verifying_key();
        let node_id = 1234;

        let task_queue_size = 10;
        let (task_sender, mut task_receiver) = mpsc::channel::<Task>(task_queue_size);

        // Run task_master in a tokio task to stay in the same thread context
        let (shutdown_sender, _) = broadcast::channel(1); // Only one shutdown signal needed
        let (event_sender, _event_receiver) = mpsc::channel::<Event>(100);
        let shutdown_receiver = shutdown_sender.subscribe();
        let task_master_handle = tokio::spawn(async move {
            fetch_prover_tasks(
                node_id,
                verifying_key,
                orchestrator_client,
                task_sender,
                event_sender,
                shutdown_receiver,
            )
            .await;
        });

        // Receive tasks
        let mut received = 0;
        for _i in 0..task_queue_size {
            match tokio::time::timeout(Duration::from_secs(2), task_receiver.recv()).await {
                Ok(Some(task)) => {
                    println!("Received task {}: {:?}", received, task);
                    received += 1;
                }
                Ok(None) => {
                    eprintln!("Channel closed unexpectedly");
                    break;
                }
                Err(_) => {
                    eprintln!("Timed out waiting for task {}", received);
                    break;
                }
            }
        }

        task_master_handle.abort();
    }
}
