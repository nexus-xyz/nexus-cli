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
    last_success_duration_secs: Option<u64>,
    last_success_difficulty: Option<crate::nexus_orchestrator::TaskDifficulty>,
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
            last_success_duration_secs: None,
            last_success_difficulty: None,
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
        // Determine desired max difficulty
        let desired = if let Some(override_diff) = self.config.max_difficulty_override {
            override_diff
        } else {
            // adaptive: start at Large by default
            let current = self
                .last_success_difficulty
                .unwrap_or(crate::nexus_orchestrator::TaskDifficulty::Large);
            // If last success took >= 7 minutes, don't increase
            let promote = !matches!(self.last_success_duration_secs, Some(secs) if secs >= 7 * 60);
            if promote {
                match current {
                    crate::nexus_orchestrator::TaskDifficulty::Small => {
                        crate::nexus_orchestrator::TaskDifficulty::Medium
                    }
                    crate::nexus_orchestrator::TaskDifficulty::Medium => {
                        crate::nexus_orchestrator::TaskDifficulty::Large
                    }
                    crate::nexus_orchestrator::TaskDifficulty::Large => {
                        // By default, do not request EXTRA_LARGE unless override is set
                        crate::nexus_orchestrator::TaskDifficulty::Large
                    }
                    other => other,
                }
            } else {
                current
            }
        };

        match self
            .network_client
            .fetch_task(
                self.orchestrator.as_ref(),
                &self.node_id.to_string(),
                self.verifying_key,
                desired,
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
