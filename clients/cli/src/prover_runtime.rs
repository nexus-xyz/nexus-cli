//! Prover Runtime
//!
//! Handles background execution of proof tasks in both authenticated and anonymous modes.
//! Spawns async workers, dispatches tasks, and reports progress back to the UI.
//!
//! Includes:
//! - Task fetching from the orchestrator (authenticated mode)
//! - Worker management and task dispatching
//! - Prover event reporting

use crate::orchestrator::{Orchestrator, OrchestratorClient};
use crate::prover::authenticated_proving;
use crate::task::Task;
use chrono::Local;
use ed25519_dalek::{SigningKey, VerifyingKey};
use nexus_sdk::stwo::seq::Proof;
use sha3::{Digest, Keccak256};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::task::JoinHandle;

/// Events emitted by prover (worker) threads.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ProverEvent {
    Message {
        worker_id: usize,
        data: String,
    },
    #[allow(unused)]
    Done {
        worker_id: usize,
    },
}

/// Starts authenticated workers that fetch tasks from the orchestrator and process them.
pub fn start_authenticated_workers(
    node_id: u64,
    signing_key: SigningKey,
    orchestrator: OrchestratorClient,
    num_workers: usize,
) -> Receiver<ProverEvent> {
    // Task fetching
    let task_queue_size = 100;
    let (task_sender, task_receiver) = channel::<Task>(task_queue_size);
    let verifying_key = signing_key.verifying_key();
    let _fetch_prover_tasks_handle = {
        let orchestrator = orchestrator.clone();
        tokio::spawn(async move {
            fetch_prover_tasks(node_id, verifying_key, Box::new(orchestrator), task_sender).await;
        })
    };

    // Workers
    let (result_sender, result_receiver) = channel::<(Task, Proof)>(1000);
    let (prover_event_sender, prover_event_receiver) = channel::<ProverEvent>(100);
    let (worker_senders, _worker_handles) =
        start_workers(num_workers, result_sender.clone(), prover_event_sender);

    // Dispatch tasks to workers
    let _dispatcher_handle = start_dispatcher(task_receiver, worker_senders);

    // Send proofs to the orchestrator
    let _submit_proofs_handle = submit_proofs(signing_key, Box::new(orchestrator), result_receiver);

    prover_event_receiver
}

/// Starts anonymous workers that repeatedly prove a program with hardcoded inputs.
pub fn start_anonymous_workers(num_workers: usize) -> Receiver<ProverEvent> {
    let (prover_event_sender, prover_event_receiver) = channel::<ProverEvent>(100);
    for worker_id in 0..num_workers {
        let prover_event_sender = prover_event_sender.clone();
        tokio::spawn(async move {
            loop {
                match crate::prover::prove_anonymously() {
                    Ok(_proof) => {
                        let now = Local::now();
                        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
                        let message = format!(
                            "✅ [{}] Anonymous proof completed successfully [Worker {}]",
                            timestamp, worker_id
                        );
                        let _ = prover_event_sender
                            .send(ProverEvent::Message {
                                worker_id,
                                data: message,
                            })
                            .await;
                    }
                    Err(e) => {
                        let message = format!("Anonymous Worker {}: Error - {}", worker_id, e);
                        let _ = prover_event_sender
                            .send(ProverEvent::Message {
                                worker_id,
                                data: message,
                            })
                            .await;
                    }
                }
                // Optional cooldown to avoid spamming
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
        });
    }

    prover_event_receiver
}

/// Fetches tasks from the orchestrator and place them in the task queue.
pub async fn fetch_prover_tasks(
    node_id: u64,
    verifying_key: VerifyingKey,
    orchestrator_client: Box<dyn Orchestrator>,
    sender: Sender<Task>,
) {
    loop {
        match orchestrator_client
            .get_proof_task(&node_id.to_string(), verifying_key)
            .await
        {
            Ok(task) => {
                if sender.send(task).await.is_err() {
                    println!("sender.send() failed, task queue is closed");
                    return;
                }
            }
            Err(_e) => {
                // TODO: Log error.
                // println!("Failed to fetch task: {}", e);
            }
        }
        // Be polite to the orchestrator and wait a bit before fetching the next task.
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

/// Submits proofs to the orchestrator or another service.
pub async fn submit_proofs(
    signing_key: SigningKey,
    orchestrator: Box<dyn Orchestrator>,
    mut results: Receiver<(Task, Proof)>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some((task, proof)) = results.recv().await {
            let proof_bytes = postcard::to_allocvec(&proof).expect("Failed to serialize proof");
            let proof_hash = format!("{:x}", Keccak256::digest(&proof_bytes));

            // TODO: Handle error
            let _res = orchestrator
                .submit_proof(&task.task_id, &proof_hash, proof_bytes, signing_key.clone())
                .await;
        }
    })
}

/// Spawns a dispatcher that forwards tasks to available workers in round-robin fashion.
pub fn start_dispatcher(
    mut task_receiver: Receiver<Task>,
    worker_senders: Vec<Sender<Task>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut next_worker = 0;
        while let Some(task) = task_receiver.recv().await {
            let target = next_worker % worker_senders.len();
            if (worker_senders[target].send(task).await).is_err() {
                eprintln!("Dispatcher: worker {} channel closed", target);
            }
            next_worker += 1;
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
    results_sender: Sender<(Task, Proof)>,
    prover_event_sender: Sender<ProverEvent>,
) -> (Vec<Sender<Task>>, Vec<JoinHandle<()>>) {
    let mut senders = Vec::with_capacity(num_workers);
    let mut handles = Vec::with_capacity(num_workers);

    for worker_id in 0..num_workers {
        let (task_sender, mut task_receiver) = tokio::sync::mpsc::channel::<Task>(8);
        let prover_event_sender = prover_event_sender.clone();
        let results_sender = results_sender.clone();
        let handle = tokio::spawn(async move {
            while let Some(task) = task_receiver.recv().await {
                let stwo_prover =
                    crate::prover::get_default_stwo_prover().expect("Failed to create prover");
                match authenticated_proving(&task, stwo_prover).await {
                    Ok(proof) => {
                        let now = Local::now();
                        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
                        let message = format!(
                            "✅ [{}] Proof completed successfully [Prover {}]",
                            timestamp, worker_id
                        );
                        let _ = prover_event_sender
                            .send(ProverEvent::Message {
                                worker_id,
                                data: message,
                            })
                            .await;

                        let _ = results_sender.send((task, proof)).await; // Send the task and proof to the results channel
                    }
                    Err(e) => {
                        let message = format!("Worker {}: Error - {}", worker_id, e);
                        let _ = prover_event_sender
                            .send(ProverEvent::Message {
                                worker_id,
                                data: message,
                            })
                            .await;
                    }
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
    use crate::prover_runtime::fetch_prover_tasks;
    use crate::task::Task;
    use std::time::Duration;
    use tokio::sync::mpsc;

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
        let task_master_handle = tokio::spawn(async move {
            fetch_prover_tasks(node_id, verifying_key, orchestrator_client, task_sender).await;
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
