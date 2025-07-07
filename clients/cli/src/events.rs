//! Event System
//!
//! Types and implementations for worker events and logging

use crate::error_classifier::LogLevel;
use crate::logging::should_log_with_env;
use chrono::Local;
use std::fmt::Display;

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
    Shutdown,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Event {
    pub worker: Worker,
    pub msg: String,
    pub timestamp: String,
    pub event_type: EventType,
    pub log_level: LogLevel,
}

impl Event {
    pub fn new(kind: Worker, msg: String, event_type: EventType) -> Self {
        Self {
            worker: kind,
            msg,
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            event_type,
            log_level: LogLevel::Info,
        }
    }

    pub fn new_with_level(
        kind: Worker,
        msg: String,
        event_type: EventType,
        log_level: LogLevel,
    ) -> Self {
        Self {
            worker: kind,
            msg,
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            event_type,
            log_level,
        }
    }

    pub fn task_fetcher(msg: String, event_type: EventType) -> Self {
        Self::new(Worker::TaskFetcher, msg, event_type)
    }

    pub fn task_fetcher_with_level(
        msg: String,
        event_type: EventType,
        log_level: LogLevel,
    ) -> Self {
        Self::new_with_level(Worker::TaskFetcher, msg, event_type, log_level)
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
        Self::new_with_level(Worker::Prover(worker_id), msg, event_type, log_level)
    }

    pub fn proof_submitter(msg: String, event_type: EventType) -> Self {
        Self::new(Worker::ProofSubmitter, msg, event_type)
    }

    pub fn proof_submitter_with_level(
        msg: String,
        event_type: EventType,
        log_level: LogLevel,
    ) -> Self {
        Self::new_with_level(Worker::ProofSubmitter, msg, event_type, log_level)
    }

    pub fn version_checker_with_level(
        msg: String,
        event_type: EventType,
        log_level: LogLevel,
    ) -> Self {
        Self::new_with_level(Worker::VersionChecker, msg, event_type, log_level)
    }

    pub fn should_display(&self) -> bool {
        // Always show success events and info level events
        if self.event_type == EventType::Success || self.log_level >= LogLevel::Info {
            return true;
        }
        should_log_with_env(self.log_level)
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let worker_type: String = match self.worker {
            Worker::TaskFetcher => "Task Fetcher".to_string(),
            Worker::Prover(worker_id) => format!("Prover {}", worker_id),
            Worker::ProofSubmitter => "Proof Submitter".to_string(),
            Worker::VersionChecker => "Version Checker".to_string(),
        };
        write!(
            f,
            "{} [{}] {}: {}",
            self.event_type, self.timestamp, worker_type, self.msg
        )
    }
}
