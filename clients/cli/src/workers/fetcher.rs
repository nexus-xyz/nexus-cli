//! Task fetching with network retry logic

use super::core::{EventSender, WorkerConfig};
use crate::analytics::track_got_task;
use crate::consts::cli_consts::{difficulty, rate_limiting, task_fetching};
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
    pub last_success_duration_secs: Option<u64>,
    pub last_success_difficulty: Option<crate::nexus_orchestrator::TaskDifficulty>,
    last_requested_difficulty: Option<crate::nexus_orchestrator::TaskDifficulty>,
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
            last_requested_difficulty: None,
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
        let desired = if let Some(override_diff) = self.config.max_difficulty {
            override_diff
        } else {
            // Adaptive difficulty system:
            // - Starts at SmallMedium by default
            // - Promotes if previous task completed in < PROMOTION_THRESHOLD_SECS
            // - Promotion path: SmallMedium → Medium → Large → ExtraLarge → ExtraLarge2
            // - Small difficulty does not auto-promote (manual override only)
            if let Some(current) = self.last_success_difficulty {
                // If last success took >= promotion threshold, don't increase difficulty
                let promote = !matches!(
                    self.last_success_duration_secs,
                    Some(secs) if secs >= difficulty::PROMOTION_THRESHOLD_SECS
                );
                if promote {
                    match current {
                        crate::nexus_orchestrator::TaskDifficulty::Small => {
                            // If server overrides to Small, promote to SmallMedium
                            // This handles server-side reputation gating
                            crate::nexus_orchestrator::TaskDifficulty::SmallMedium
                        }
                        crate::nexus_orchestrator::TaskDifficulty::SmallMedium => {
                            crate::nexus_orchestrator::TaskDifficulty::Medium
                        }
                        crate::nexus_orchestrator::TaskDifficulty::Medium => {
                            crate::nexus_orchestrator::TaskDifficulty::Large
                        }
                        crate::nexus_orchestrator::TaskDifficulty::Large => {
                            crate::nexus_orchestrator::TaskDifficulty::ExtraLarge
                        }
                        crate::nexus_orchestrator::TaskDifficulty::ExtraLarge => {
                            crate::nexus_orchestrator::TaskDifficulty::ExtraLarge2
                        }
                        crate::nexus_orchestrator::TaskDifficulty::ExtraLarge2 => {
                            // Already at maximum difficulty
                            crate::nexus_orchestrator::TaskDifficulty::ExtraLarge2
                        }
                    }
                } else {
                    current
                }
            } else {
                // No previous success - start at SmallMedium
                crate::nexus_orchestrator::TaskDifficulty::SmallMedium
            }
        };

        // Log the difficulty we're requesting vs what we receive
        let requested_difficulty = desired;

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
            Ok(proof_task_result) => {
                // Log difficulty adjustment if server overrides our request
                if proof_task_result.actual_difficulty != requested_difficulty {
                    self.event_sender
                        .send_task_event(
                            format!(
                                "Server adjusted difficulty: requested {:?}, assigned {:?} (reputation gating)",
                                requested_difficulty,
                                proof_task_result.actual_difficulty
                            ),
                            EventType::Success,
                            LogLevel::Info,
                        )
                        .await;
                }

                // Log successful fetch
                self.event_sender
                    .send_task_event(
                        format!("Step 1 of 4: Got task {}", proof_task_result.task.task_id),
                        EventType::Success,
                        LogLevel::Info,
                    )
                    .await;

                // Track analytics for successful fetch
                tokio::spawn(track_got_task(
                    proof_task_result.task.clone(),
                    self.config.environment.clone(),
                    self.config.client_id.clone(),
                ));

                // Store the actual difficulty received from server for success tracking
                self.last_requested_difficulty = Some(proof_task_result.actual_difficulty);

                Ok(proof_task_result.task)
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

    /// Update success tracking after completing a task
    /// Uses the actual difficulty received from the server
    pub fn update_success_tracking(&mut self, duration_secs: u64) {
        if let Some(difficulty) = self.last_requested_difficulty {
            self.last_success_difficulty = Some(difficulty);
            self.last_success_duration_secs = Some(duration_secs);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;
    use crate::orchestrator::error::OrchestratorError;
    use crate::task::Task;
    use crate::workers::core::WorkerConfig;
    use ed25519_dalek::{SigningKey, VerifyingKey};
    use tokio::sync::mpsc;

    // Mock orchestrator for testing
    struct MockOrchestrator;

    impl MockOrchestrator {
        fn new() -> Self {
            Self
        }
    }

    #[async_trait::async_trait]
    impl Orchestrator for MockOrchestrator {
        async fn get_proof_task(
            &self,
            _node_id: &str,
            _verifying_key: VerifyingKey,
            max_difficulty: crate::nexus_orchestrator::TaskDifficulty,
        ) -> Result<crate::orchestrator::client::ProofTaskResult, OrchestratorError> {
            // Return a mock task with the requested difficulty as actual difficulty
            let task = Task {
                task_id: "test_task".to_string(),
                program_id: "test_program".to_string(),
                public_inputs: vec![1, 2, 3],
                public_inputs_list: vec![vec![1, 2, 3]],
                task_type: crate::nexus_orchestrator::TaskType::ProofHash,
                difficulty: crate::nexus_orchestrator::TaskDifficulty::Medium,
            };

            Ok(crate::orchestrator::client::ProofTaskResult {
                task,
                actual_difficulty: max_difficulty,
            })
        }

        fn environment(&self) -> &Environment {
            &Environment::Production
        }

        async fn get_user(&self, _wallet_address: &str) -> Result<String, OrchestratorError> {
            Ok("test_user".to_string())
        }

        async fn register_user(
            &self,
            _user_id: &str,
            _wallet_address: &str,
        ) -> Result<(), OrchestratorError> {
            Ok(())
        }

        async fn register_node(&self, _user_id: &str) -> Result<String, OrchestratorError> {
            Ok("test_node".to_string())
        }

        async fn submit_proof(
            &self,
            _task_id: &str,
            _proof_hash: &str,
            _proof: Vec<u8>,
            _proofs: Vec<Vec<u8>>,
            _signing_key: SigningKey,
            _num_provers: usize,
            _task_type: crate::nexus_orchestrator::TaskType,
            _individual_proof_hashes: &[String],
        ) -> Result<(), OrchestratorError> {
            Ok(())
        }

        async fn get_node(&self, _node_id: &str) -> Result<String, OrchestratorError> {
            Ok("test_node".to_string())
        }
    }

    fn create_test_fetcher() -> TaskFetcher {
        let (event_sender, _event_receiver) = mpsc::channel(100);
        let event_sender = crate::workers::core::EventSender::new(event_sender);
        let config = WorkerConfig::new(Environment::Production, "test_client".to_string());

        TaskFetcher::new(
            12345,
            VerifyingKey::from_bytes(&[0u8; 32])
                .expect("failed to construct VerifyingKey from bytes"),
            Box::new(MockOrchestrator::new()),
            event_sender,
            &config,
        )
    }

    #[tokio::test]
    async fn test_default_difficulty_is_small_medium() {
        let mut fetcher = create_test_fetcher();

        // First fetch should default to SmallMedium
        let task = fetcher
            .fetch_task()
            .await
            .expect("fetcher.fetch_task failed");
        assert_eq!(task.task_id, "test_task");

        // Verify the last requested difficulty was SmallMedium
        assert_eq!(
            fetcher.last_requested_difficulty,
            Some(crate::nexus_orchestrator::TaskDifficulty::SmallMedium)
        );
    }

    #[tokio::test]
    async fn test_small_does_not_promote_automatically() {
        let mut fetcher = create_test_fetcher();

        // Set up initial state: last success was Small
        fetcher.last_success_difficulty = Some(crate::nexus_orchestrator::TaskDifficulty::Small);
        fetcher.last_success_duration_secs = Some(300); // 5 minutes - would normally promote

        let task = fetcher
            .fetch_task()
            .await
            .expect("fetcher.fetch_task failed");
        assert_eq!(task.task_id, "test_task");

        // Should NOT promote from Small (stays at Small)
        assert_eq!(
            fetcher.last_requested_difficulty,
            Some(crate::nexus_orchestrator::TaskDifficulty::Small)
        );
    }

    #[tokio::test]
    async fn test_promotion_path_small_medium_to_medium() {
        let mut fetcher = create_test_fetcher();

        // Set up initial state: last success was SmallMedium
        fetcher.last_success_difficulty =
            Some(crate::nexus_orchestrator::TaskDifficulty::SmallMedium);
        fetcher.last_success_duration_secs = Some(300); // 5 minutes - should promote

        let task = fetcher
            .fetch_task()
            .await
            .expect("fetcher.fetch_task failed");
        assert_eq!(task.task_id, "test_task");

        // Should promote from SmallMedium to Medium
        assert_eq!(
            fetcher.last_requested_difficulty,
            Some(crate::nexus_orchestrator::TaskDifficulty::Medium)
        );
    }

    #[tokio::test]
    async fn test_promotion_path_medium_to_large() {
        let mut fetcher = create_test_fetcher();

        // Set up initial state: last success was Medium
        fetcher.last_success_difficulty = Some(crate::nexus_orchestrator::TaskDifficulty::Medium);
        fetcher.last_success_duration_secs = Some(300); // 5 minutes - should promote

        let task = fetcher
            .fetch_task()
            .await
            .expect("fetcher.fetch_task failed");
        assert_eq!(task.task_id, "test_task");

        // Should promote from Medium to Large
        assert_eq!(
            fetcher.last_requested_difficulty,
            Some(crate::nexus_orchestrator::TaskDifficulty::Large)
        );
    }

    #[tokio::test]
    async fn test_large_promotes_to_extra_large() {
        let mut fetcher = create_test_fetcher();

        // Set up initial state: last success was Large
        fetcher.last_success_difficulty = Some(crate::nexus_orchestrator::TaskDifficulty::Large);
        fetcher.last_success_duration_secs = Some(300); // 5 minutes - should promote

        let task = fetcher
            .fetch_task()
            .await
            .expect("fetcher.fetch_task failed");
        assert_eq!(task.task_id, "test_task");

        // Should promote from Large to ExtraLarge
        assert_eq!(
            fetcher.last_requested_difficulty,
            Some(crate::nexus_orchestrator::TaskDifficulty::ExtraLarge)
        );
    }

    #[tokio::test]
    async fn test_no_promotion_when_task_takes_too_long() {
        let mut fetcher = create_test_fetcher();

        // Set up initial state: last success was Medium, but took 8 minutes (too long)
        fetcher.last_success_difficulty = Some(crate::nexus_orchestrator::TaskDifficulty::Medium);
        fetcher.last_success_duration_secs = Some(480); // 8 minutes - should NOT promote

        let task = fetcher
            .fetch_task()
            .await
            .expect("fetcher.fetch_task failed");
        assert_eq!(task.task_id, "test_task");

        // Should NOT promote (stays at Medium)
        assert_eq!(
            fetcher.last_requested_difficulty,
            Some(crate::nexus_orchestrator::TaskDifficulty::Medium)
        );
    }

    #[tokio::test]
    async fn test_manual_override_works() {
        let mut fetcher = create_test_fetcher();

        // Set up manual override to ExtraLarge
        fetcher.config.max_difficulty = Some(crate::nexus_orchestrator::TaskDifficulty::ExtraLarge);

        let task = fetcher
            .fetch_task()
            .await
            .expect("fetcher.fetch_task failed");
        assert_eq!(task.task_id, "test_task");

        // Should use the manual override (ExtraLarge)
        assert_eq!(
            fetcher.last_requested_difficulty,
            Some(crate::nexus_orchestrator::TaskDifficulty::ExtraLarge)
        );
    }

    #[tokio::test]
    async fn test_manual_override_to_small() {
        let mut fetcher = create_test_fetcher();

        // Set up manual override to Small
        fetcher.config.max_difficulty = Some(crate::nexus_orchestrator::TaskDifficulty::Small);

        let task = fetcher
            .fetch_task()
            .await
            .expect("fetcher.fetch_task failed");
        assert_eq!(task.task_id, "test_task");

        // Should use the manual override (Small)
        assert_eq!(
            fetcher.last_requested_difficulty,
            Some(crate::nexus_orchestrator::TaskDifficulty::Small)
        );
    }

    #[tokio::test]
    async fn test_success_tracking_update() {
        let mut fetcher = create_test_fetcher();

        // Initially no success tracking
        assert_eq!(fetcher.last_success_difficulty, None);
        assert_eq!(fetcher.last_success_duration_secs, None);

        // Set a requested difficulty
        fetcher.last_requested_difficulty = Some(crate::nexus_orchestrator::TaskDifficulty::Medium);

        // Update success tracking
        fetcher.update_success_tracking(300); // 5 minutes

        // Verify tracking was updated
        assert_eq!(
            fetcher.last_success_difficulty,
            Some(crate::nexus_orchestrator::TaskDifficulty::Medium)
        );
        assert_eq!(fetcher.last_success_duration_secs, Some(300));
    }

    #[tokio::test]
    async fn test_success_tracking_without_requested_difficulty() {
        let mut fetcher = create_test_fetcher();

        // No requested difficulty set
        fetcher.last_requested_difficulty = None;

        // Update success tracking
        fetcher.update_success_tracking(300);

        // Should not update tracking when no requested difficulty
        assert_eq!(fetcher.last_success_difficulty, None);
        assert_eq!(fetcher.last_success_duration_secs, None);
    }

    #[tokio::test]
    async fn test_extra_large_promotes_to_extra_large2() {
        let mut fetcher = create_test_fetcher();

        // Set up initial state: last success was ExtraLarge
        fetcher.last_success_difficulty =
            Some(crate::nexus_orchestrator::TaskDifficulty::ExtraLarge);
        fetcher.last_success_duration_secs = Some(300); // 5 minutes - should promote

        let task = fetcher
            .fetch_task()
            .await
            .expect("fetcher.fetch_task failed");
        assert_eq!(task.task_id, "test_task");

        // Should promote from ExtraLarge to ExtraLarge2
        assert_eq!(
            fetcher.last_requested_difficulty,
            Some(crate::nexus_orchestrator::TaskDifficulty::ExtraLarge2)
        );
    }

    #[tokio::test]
    async fn test_extra_large2_stays_at_maximum() {
        let mut fetcher = create_test_fetcher();

        // Set up initial state: last success was ExtraLarge2 (maximum difficulty)
        fetcher.last_success_difficulty =
            Some(crate::nexus_orchestrator::TaskDifficulty::ExtraLarge2);
        fetcher.last_success_duration_secs = Some(300); // 5 minutes - would normally promote

        let task = fetcher.fetch_task().await.unwrap();
        assert_eq!(task.task_id, "test_task");

        // Should stay at ExtraLarge2 (maximum difficulty reached)
        assert_eq!(
            fetcher.last_requested_difficulty,
            Some(crate::nexus_orchestrator::TaskDifficulty::ExtraLarge2)
        );
    }

    #[tokio::test]
    async fn test_promotion_threshold_edge_case() {
        let mut fetcher = create_test_fetcher();

        // Test exactly 7 minutes (420 seconds) - should NOT promote
        fetcher.last_success_difficulty = Some(crate::nexus_orchestrator::TaskDifficulty::Medium);
        fetcher.last_success_duration_secs = Some(420); // Exactly 7 minutes

        let task = fetcher
            .fetch_task()
            .await
            .expect("fetcher.fetch_task failed");
        assert_eq!(task.task_id, "test_task");

        // Should NOT promote (stays at Medium)
        assert_eq!(
            fetcher.last_requested_difficulty,
            Some(crate::nexus_orchestrator::TaskDifficulty::Medium)
        );
    }

    #[tokio::test]
    async fn test_promotion_threshold_just_under() {
        let mut fetcher = create_test_fetcher();

        // Test just under 7 minutes (419 seconds) - should promote
        fetcher.last_success_difficulty = Some(crate::nexus_orchestrator::TaskDifficulty::Medium);
        fetcher.last_success_duration_secs = Some(419); // Just under 7 minutes

        let task = fetcher
            .fetch_task()
            .await
            .expect("fetcher.fetch_task failed");
        assert_eq!(task.task_id, "test_task");

        // Should promote from Medium to Large
        assert_eq!(
            fetcher.last_requested_difficulty,
            Some(crate::nexus_orchestrator::TaskDifficulty::Large)
        );
    }
}
