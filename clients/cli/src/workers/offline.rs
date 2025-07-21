//! Offline Workers
//!
//! Handles local compute operations that don't require network access:
//! - Task dispatching to workers
//! - Proof computation (authenticated and anonymous)
//! - Worker management

use crate::analytics::{track_anonymous_proof_analytics, track_authenticated_proof_analytics};
use crate::environment::Environment;
use crate::error_classifier::ErrorClassifier;
use crate::events::{Event, EventType};
use crate::prover::authenticated_proving;
use crate::task::Task;
use nexus_sdk::stwo::seq::Proof;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

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
    environment: Environment,
    client_id: String,
) -> (Vec<mpsc::Sender<Task>>, Vec<JoinHandle<()>>) {
    let mut senders = Vec::with_capacity(num_workers);
    let mut handles = Vec::with_capacity(num_workers);

    for worker_id in 0..num_workers {
        let (task_sender, mut task_receiver) = mpsc::channel::<Task>(8);
        // Clone senders and receivers for each worker.
        let prover_event_sender = event_sender.clone();
        let results_sender = results_sender.clone();
        let mut shutdown_rx = shutdown.resubscribe();
        let client_id = client_id.clone();
        let environment = environment.clone();
        let error_classifier = ErrorClassifier::new();
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
                    // Check if there are tasks to process
                    Some(task) = task_receiver.recv() => {
                        match authenticated_proving(&task, &environment, &client_id).await {
                            Ok(proof) => {
                                let message = format!(
                                    "[Task step 2 of 3] Proof completed successfully (Task ID: {})",
                                    task.task_id
                                );
                                let _ = prover_event_sender
                                    .send(Event::prover(worker_id, message, EventType::Success))
                                    .await;

                                // Track analytics for successful proof (non-blocking)
                                tokio::spawn(track_authenticated_proof_analytics(task.clone(), environment.clone(), client_id.clone()));

                                let _ = results_sender.send((task, proof)).await;
                            }
                            Err(e) => {
                                let log_level = error_classifier.classify_worker_error(&e);
                                let message = format!("Error: {}", e);
                                let event = Event::prover_with_level(worker_id, message, EventType::Error, log_level);
                                if event.should_display() {
                                    let _ = prover_event_sender.send(event).await;
                                }

                                // For analytics errors, continue processing but don't send result
                                // For other errors, also don't send result (task failed)
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

/// Starts anonymous workers that repeatedly prove a program with hardcoded inputs.
pub async fn start_anonymous_workers(
    num_workers: usize,
    shutdown: broadcast::Receiver<()>,
    environment: Environment,
    client_id: String,
) -> (mpsc::Receiver<Event>, Vec<JoinHandle<()>>) {
    let (event_sender, event_receiver) = mpsc::channel::<Event>(100);
    let mut join_handles = Vec::new();
    for worker_id in 0..num_workers {
        let prover_event_sender = event_sender.clone();
        let mut shutdown_rx = shutdown.resubscribe(); // clone receiver for each worker
        let client_id = client_id.clone();
        let environment = environment.clone();
        let error_classifier = ErrorClassifier::new();

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
                        match crate::prover::prove_anonymously().await {
                            Ok(_proof) => {
                                let message = "Anonymous proof completed successfully".to_string();
                                let _ = prover_event_sender
                                    .send(Event::prover(worker_id, message, EventType::Success)).await;

                                // Track analytics for successful anonymous proof (non-blocking)
                                tokio::spawn(track_anonymous_proof_analytics(environment.clone(), client_id.clone()));
                            }
                            Err(e) => {
                                let log_level = error_classifier.classify_worker_error(&e);
                                let message = format!("Anonymous Worker: Error - {}", e);
                                let event = Event::prover_with_level(worker_id, message, EventType::Error, log_level);
                                if event.should_display() {
                                    let _ = prover_event_sender.send(event).await;
                                }

                                // For analytics errors, this is non-critical, continue the loop
                                // For other errors, also continue (anonymous mode keeps retrying)
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
