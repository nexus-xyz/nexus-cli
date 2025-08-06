//! Event System
//!
//! Types and implementations for worker events and logging

use crate::error_classifier::LogLevel;
use crate::logging::should_log_with_env;
use chrono::Local;
use std::fmt::Display;
use std::time::Instant;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Worker {
    /// Worker that fetches tasks from the orchestrator and processes them.
    TaskFetcher,
    /// Worker that performs proving tasks.
    Prover(usize),
    /// Worker that submits proofs to the orchestrator.
    ProofSubmitter,
    /// Worker that checks for new CLI versions.
    VersionChecker,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display)]
pub enum EventType {
    Success,
    Error,
    Refresh,
    Waiting,
    Shutdown,
    StateChange,
}

/// Represents the current state in the proof pipeline
#[derive(Debug, Copy, Clone, Eq, PartialEq, strum::Display)]
pub enum ProverState {
    /// Fetching a new task from the orchestrator
    Fetching,
    /// Computing the proof
    Proving,
    /// Submitting the proof to the orchestrator
    Submitting,
    /// Waiting before fetching next task (idle state)
    Waiting,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub worker: Worker,
    pub msg: String,
    pub timestamp: String,
    pub event_type: EventType,
    pub log_level: LogLevel,
    /// Optional state information for state change events
    pub prover_state: Option<ProverState>,
    /// Optional timer for state change events (when the state started)
    pub state_start_time: Option<Instant>,
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.worker == other.worker
            && self.msg == other.msg
            && self.timestamp == other.timestamp
            && self.event_type == other.event_type
            && self.log_level == other.log_level
            && self.prover_state == other.prover_state
        // Note: We don't compare state_start_time since Instant doesn't implement Eq
    }
}

impl Eq for Event {}

impl Event {
    fn new_base(worker: Worker, msg: String, event_type: EventType, log_level: LogLevel) -> Self {
        Self {
            worker,
            msg,
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            event_type,
            log_level,
            prover_state: None,
            state_start_time: None,
        }
    }

    pub fn new(worker: Worker, msg: String, event_type: EventType) -> Self {
        Self::new_base(worker, msg, event_type, LogLevel::Info)
    }

    pub fn state_change(state: ProverState, msg: String, timer: Instant) -> Self {
        Self {
            worker: Worker::TaskFetcher,
            msg,
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            event_type: EventType::StateChange,
            log_level: LogLevel::Info,
            prover_state: Some(state),
            state_start_time: Some(timer),
        }
    }

    pub fn task_fetcher_with_level(
        msg: String,
        event_type: EventType,
        log_level: LogLevel,
    ) -> Self {
        Self::new_base(Worker::TaskFetcher, msg, event_type, log_level)
    }

    pub fn prover(worker_id: usize, msg: String, event_type: EventType) -> Self {
        Self::new(Worker::Prover(worker_id), msg, event_type)
    }

    pub fn prover_with_level(
        worker_id: usize,
        msg: String,
        event_type: EventType,
        log_level: LogLevel,
    ) -> Self {
        Self::new_base(Worker::Prover(worker_id), msg, event_type, log_level)
    }

    pub fn proof_submitter_with_level(
        msg: String,
        event_type: EventType,
        log_level: LogLevel,
    ) -> Self {
        Self::new_base(Worker::ProofSubmitter, msg, event_type, log_level)
    }

    pub fn version_checker_with_level(
        msg: String,
        event_type: EventType,
        log_level: LogLevel,
    ) -> Self {
        Self::new_base(Worker::VersionChecker, msg, event_type, log_level)
    }

    pub fn should_display(&self) -> bool {
        // Always show success events and info level events
        if self.event_type == EventType::Success || self.log_level >= LogLevel::Info {
            return true;
        }
        // StateChange events should be handled separately (not displayed in logs)
        if self.event_type == EventType::StateChange {
            return false;
        }
        should_log_with_env(self.log_level)
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}] {}", self.event_type, self.timestamp, self.msg)
    }
}
