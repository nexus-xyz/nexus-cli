//! Core worker utilities and traits

use crate::events::{Event, EventType};
use crate::logging::LogLevel;
use tokio::sync::mpsc;

/// Common event sending utilities for workers
#[derive(Clone)]
pub struct EventSender {
    sender: mpsc::Sender<Event>,
}

impl EventSender {
    pub fn new(sender: mpsc::Sender<Event>) -> Self {
        Self { sender }
    }

    /// Send a generic event
    pub async fn send_event(&self, event: Event) {
        let _ = self.sender.send(event).await;
    }

    pub async fn send_task_event(
        &self,
        message: String,
        event_type: EventType,
        log_level: LogLevel,
    ) {
        let _ = self
            .sender
            .send(Event::task_fetcher_with_level(
                message, event_type, log_level,
            ))
            .await;
    }

    pub async fn send_proof_event(
        &self,
        message: String,
        event_type: EventType,
        log_level: LogLevel,
    ) {
        let _ = self
            .sender
            .send(Event::proof_submitter_with_level(
                message, event_type, log_level,
            ))
            .await;
    }

    pub async fn send_prover_event(
        &self,
        thread_id: usize,
        message: String,
        event_type: EventType,
        log_level: LogLevel,
    ) {
        let _ = self
            .sender
            .send(Event::prover_with_level(
                thread_id, message, event_type, log_level,
            ))
            .await;
    }
}

/// Worker configuration shared across all worker types
#[derive(Clone)]
pub struct WorkerConfig {
    pub environment: crate::environment::Environment,
    pub client_id: String,
    pub max_difficulty: Option<crate::nexus_orchestrator::TaskDifficulty>,
    pub num_workers: usize,
}

impl WorkerConfig {
    pub fn new(environment: crate::environment::Environment, client_id: String) -> Self {
        Self {
            environment,
            client_id,
            max_difficulty: None,
            num_workers: 1,
        }
    }
}
