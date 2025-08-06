//! Task fetching with network retry logic

use super::core::EventSender;
use crate::consts::cli_consts::{task_fetching, rate_limiting};
use crate::error_classifier::LogLevel;
use crate::events::EventType;
use crate::network::{NetworkClient, RequestTimer, RequestTimerConfig};
use crate::orchestrator::Orchestrator;
use crate::task::Task;
use ed25519_dalek::VerifyingKey;
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("Network error: {0}")]
    Network(#[from] crate::orchestrator::error::OrchestratorError),
}

/// Task fetcher with built-in retry and error handling
pub struct TaskFetcher {
    node_id: u64,
    verifying_key: VerifyingKey,
    orchestrator: Box<dyn Orchestrator>,
    network_client: NetworkClient,
    event_sender: EventSender,
}

impl TaskFetcher {
    pub fn new(
        node_id: u64,
        verifying_key: VerifyingKey,
        orchestrator: Box<dyn Orchestrator>,
        event_sender: EventSender,
    ) -> Self {
        // Configure request timer for task fetching
        let timer_config = RequestTimerConfig::combined(
            task_fetching::rate_limit_interval(),
            rate_limiting::TASK_FETCH_MAX_REQUESTS_PER_WINDOW,
            rate_limiting::task_fetch_window(),
            task_fetching::initial_backoff(), // Use as default retry delay
        );
        let request_timer = RequestTimer::new(timer_config);

        // Create network client with retry logic
        let network_client = NetworkClient::new(request_timer, task_fetching::MAX_RETRIES);

        Self {
            node_id,
            verifying_key,
            orchestrator,
            network_client,
            event_sender,
        }
    }

    /// Fetch a single task with automatic retry and proper logging
    pub async fn fetch_task(&mut self) -> Result<Task, FetchError> {
        // Wait until we can proceed
        while !self.network_client.request_timer_mut().can_proceed() {
            let wait_time = self.network_client.request_timer_mut().time_until_next();
            if wait_time > Duration::ZERO {
                sleep(wait_time).await;
            }
        }

        // Log start of fetching
        self.event_sender.send_task_event(
            "Step 1 of 4: Fetching task...".to_string(),
            EventType::Success,
            LogLevel::Info,
        ).await;

        // Attempt to fetch task through network client
        match self.network_client.fetch_task(
            self.orchestrator.as_ref(),
            &self.node_id.to_string(),
            self.verifying_key,
        ).await {
            Ok(task) => {
                // Log successful fetch
                self.event_sender.send_task_event(
                    format!("Step 1 of 4: Got task {}", task.task_id),
                    EventType::Success,
                    LogLevel::Info,
                ).await;

                Ok(task)
            }
            Err(e) => {
                // Log fetch failure with appropriate level
                let log_level = self.network_client.classify_error(&e);
                self.event_sender.send_task_event(
                    format!("Failed to fetch task: {}", e),
                    EventType::Error,
                    log_level,
                ).await;

                Err(FetchError::Network(e))
            }
        }
    }
}
