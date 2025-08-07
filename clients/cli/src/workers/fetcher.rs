//! Task fetching with network retry logic

use super::core::{EventSender, WorkerConfig};
use crate::analytics::track_got_task;
use crate::consts::cli_consts::{rate_limiting, task_fetching};
use crate::events::EventType;
use crate::logging::LogLevel;
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
    config: WorkerConfig,
}

impl TaskFetcher {
    pub fn new(
        node_id: u64,
        verifying_key: VerifyingKey,
        orchestrator: Box<dyn Orchestrator>,
        event_sender: EventSender,
        config: &WorkerConfig,
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
            config: config.clone(),
        }
    }

    /// Fetch a single task with automatic retry and proper logging
    pub async fn fetch_task(&mut self) -> Result<Task, FetchError> {
        // Check if we can proceed immediately
        let can_proceed_immediately = self.network_client.request_timer_mut().can_proceed();

        if can_proceed_immediately {
            self.event_sender
                .send_task_event(
                    "Step 1 of 4: Fetching task...".to_string(),
                    EventType::Refresh,
                    LogLevel::Info,
                )
                .await;
        }

        // Wait until we can proceed with accurate timing
        while !self.network_client.request_timer_mut().can_proceed() {
            let wait_time = self.network_client.request_timer_mut().time_until_next();
            if wait_time > Duration::ZERO {
                // Log the accurate wait time here
                self.event_sender
                    .send_task_event(
                        format!(
                            "Step 1 of 4: Waiting - ready for next task ({}) seconds",
                            wait_time.as_secs()
                        ),
                        EventType::Waiting,
                        LogLevel::Info,
                    )
                    .await;
                sleep(wait_time).await;
            }
        }

        // Attempt to fetch task through network client
        match self
            .network_client
            .fetch_task(
                self.orchestrator.as_ref(),
                &self.node_id.to_string(),
                self.verifying_key,
            )
            .await
        {
            Ok(task) => {
                // Log successful fetch
                self.event_sender
                    .send_task_event(
                        format!("Step 1 of 4: Got task {}", task.task_id),
                        EventType::Success,
                        LogLevel::Info,
                    )
                    .await;

                // Track analytics for successful fetch
                tokio::spawn(track_got_task(
                    task.clone(),
                    self.config.environment.clone(),
                    self.config.client_id.clone(),
                ));

                Ok(task)
            }
            Err(e) => {
                // Log fetch failure with appropriate level
                let log_level = self.network_client.classify_error(&e);
                self.event_sender
                    .send_task_event(
                        format!("Failed to fetch task: {}", e),
                        EventType::Error,
                        log_level,
                    )
                    .await;

                Err(FetchError::Network(e))
            }
        }
    }
}
