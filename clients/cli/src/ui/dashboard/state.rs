//! Dashboard state management
//!
//! Contains the main dashboard state struct and related enums

use crate::environment::Environment;
use crate::events::{Event as WorkerEvent, ProverState};
use crate::ui::metrics::{SystemMetrics, TaskFetchInfo, ZkVMMetrics};
use crate::ui::stages::ProverStage;
use std::collections::VecDeque;
use std::time::Instant;
use sysinfo::System;

/// State for tracking fetching operations
#[derive(Debug, Clone)]
pub enum FetchingState {
    Idle,
    Active { started_at: Instant },
    Timeout,
}

/// State for tracking proving operations
#[derive(Debug, Clone)]
pub enum ProvingState {
    Idle,
    Active {
        started_at: Instant,
        task_id: Option<String>,
    },
}

/// Enhanced dashboard state with real-time metrics and animations.
#[derive(Debug)]
pub struct DashboardState {
    /// Unique identifier for the node.
    pub node_id: Option<u64>,
    /// The environment in which the application is running.
    pub environment: Environment,
    /// The start time of the application, used for computing uptime.
    pub start_time: Instant,
    /// The current task being executed by the node, if any.
    pub current_task: Option<String>,
    /// Total number of (virtual) CPU cores available on the machine.
    pub total_cores: usize,
    /// Total RAM available on the machine, in GB.
    pub total_ram_gb: f64,
    /// Number of worker threads being used for proving.
    pub num_threads: usize,
    /// A queue of events received from worker threads.
    pub events: VecDeque<WorkerEvent>,
    /// Whether a new version is available.
    pub update_available: bool,
    /// The latest version string, if known.
    pub latest_version: Option<String>,
    /// Whether to disable background colors
    pub no_background_color: bool,
    /// Current prover stage and animation state
    pub prover_stage: ProverStage,
    /// System metrics (CPU, RAM, etc.)
    pub system_metrics: SystemMetrics,
    /// zkVM task metrics
    pub zkvm_metrics: ZkVMMetrics,
    /// Task fetch information for accurate timing
    pub task_fetch_info: TaskFetchInfo,
    /// Animation tick counter
    pub tick: usize,
    /// Last rate limiting message seen
    last_rate_limit_message: Option<String>,
    /// Tick when last rate limiting event was seen
    last_rate_limit_tick: Option<usize>,
    /// Timestamp of last successful proof submission
    last_submission_timestamp: Option<String>,
    /// Current fetching state (active, timeout, idle)
    fetching_state: FetchingState,
    /// Current proving state (active, idle)
    proving_state: ProvingState,
    /// Persistent system info instance for accurate CPU measurements
    sysinfo: System,
    /// Current prover state from state events
    current_prover_state: ProverState,
}

impl DashboardState {
    /// Creates a new instance of the dashboard state.
    pub fn new(
        node_id: Option<u64>,
        environment: Environment,
        start_time: Instant,
        events: &VecDeque<WorkerEvent>,
        no_background_color: bool,
        num_threads: usize,
    ) -> Self {
        // Check for version update messages in recent events
        let (update_available, latest_version, _) = Self::check_for_version_updates(events);

        Self {
            node_id,
            environment,
            start_time,
            current_task: None,
            total_cores: crate::system::num_cores(),
            total_ram_gb: crate::system::total_memory_gb(),
            num_threads,
            events: events.clone(),
            update_available,
            latest_version,
            no_background_color,
            prover_stage: ProverStage::default(),
            system_metrics: SystemMetrics::default(),
            zkvm_metrics: ZkVMMetrics::default(),
            task_fetch_info: TaskFetchInfo::default(),
            tick: 0,
            last_rate_limit_message: None,
            last_rate_limit_tick: None,
            last_submission_timestamp: None,
            fetching_state: FetchingState::Idle,
            proving_state: ProvingState::Idle,
            sysinfo: System::new_all(), // Initialize with all data for first refresh
            current_prover_state: ProverState::Waiting,
        }
    }

    /// Check recent events for version update information
    fn check_for_version_updates(
        events: &VecDeque<WorkerEvent>,
    ) -> (
        bool,
        Option<String>,
        Option<crate::version::ConstraintType>,
    ) {
        // Look for the most recent version checker event
        for event in events.iter().rev() {
            if matches!(event.worker, crate::events::Worker::VersionChecker) {
                return (true, None, None);
            }
        }
        (false, None, None)
    }

    // Getter methods for private fields
    pub fn fetching_state(&self) -> &FetchingState {
        &self.fetching_state
    }

    pub fn proving_state(&self) -> &ProvingState {
        &self.proving_state
    }

    pub fn current_prover_state(&self) -> ProverState {
        self.current_prover_state
    }

    pub fn last_submission_timestamp(&self) -> &Option<String> {
        &self.last_submission_timestamp
    }

    pub fn last_rate_limit_message(&self) -> &Option<String> {
        &self.last_rate_limit_message
    }

    pub fn last_rate_limit_tick(&self) -> Option<usize> {
        self.last_rate_limit_tick
    }

    // Setter methods for private fields (for updaters)
    pub fn set_fetching_state(&mut self, state: FetchingState) {
        self.fetching_state = state;
    }

    pub fn set_proving_state(&mut self, state: ProvingState) {
        self.proving_state = state;
    }

    pub fn set_current_prover_state(&mut self, state: ProverState) {
        self.current_prover_state = state;
    }

    pub fn set_last_submission_timestamp(&mut self, timestamp: Option<String>) {
        self.last_submission_timestamp = timestamp;
    }

    pub fn set_last_rate_limit_message(&mut self, message: Option<String>) {
        self.last_rate_limit_message = message;
    }

    pub fn set_last_rate_limit_tick(&mut self, tick: Option<usize>) {
        self.last_rate_limit_tick = tick;
    }

    pub fn get_sysinfo_mut(&mut self) -> &mut System {
        &mut self.sysinfo
    }
}
