//! Dashboard state update logic
//!
//! Contains all methods for updating dashboard state from events

use super::state::{DashboardState, FetchingState, ProvingState};
use super::utils::extract_task_id_from_message;
use crate::events::{EventType, Worker};
use crate::ui::metrics::{SystemMetrics, TaskFetchInfo, ZkVMMetrics};
use crate::ui::stages::ProverStage;
use std::time::Instant;

impl DashboardState {
    /// Update the dashboard state with new tick and metrics.
    pub fn update(&mut self) {
        self.tick += 1;

        // Update current task from recent events
        self.update_current_task();

        // Update prover stage based on recent events and task fetch info
        self.prover_stage = ProverStage::update_from_events(
            &self.events,
            &self.prover_stage,
            self.tick,
            &self.task_fetch_info,
        );

        // Update system metrics using persistent sysinfo instance for accurate CPU measurements
        let previous_peak = self.system_metrics.peak_ram_bytes;
        let previous_metrics = self.system_metrics.clone();
        self.system_metrics = SystemMetrics::update(
            self.get_sysinfo_mut(),
            previous_peak,
            Some(&previous_metrics),
        );

        // Update zkVM metrics from events
        self.update_zkvm_metrics();

        // Update task fetch info from recent events (simplified version)
        self.update_task_fetch_info();

        // Update version information from recent events
        self.update_version_info();

        // Update fetching and proving states
        self.update_fetching_state();
        self.update_proving_state();

        // Update current prover state from state events
        self.update_prover_state();
    }

    /// Update task fetch info from recent events (simplified version).
    /// In a real implementation, this would be passed from the TaskFetchState.
    fn update_task_fetch_info(&mut self) {
        // Collect events first to avoid borrowing issues
        let recent_events: Vec<_> = self.events.iter().rev().take(5).cloned().collect();

        // Look for the most recent rate limiting event
        for event in recent_events {
            if matches!(event.worker, Worker::TaskFetcher) && event.msg.contains("retrying in") {
                // Check if this is a new rate limiting message
                let current_message = event.msg.clone();
                if self.last_rate_limit_message().as_ref() != Some(&current_message) {
                    // New rate limiting event - reset the timer
                    self.set_last_rate_limit_message(Some(current_message.clone()));
                    self.set_last_rate_limit_tick(Some(self.tick));
                }

                // Extract backoff time from message like "retrying in 45s"
                if let Some(time_part) = event.msg.split("retrying in ").nth(1) {
                    if let Some(time_str) = time_part.split('s').next() {
                        if let Ok(backoff_secs) = time_str.parse::<u64>() {
                            // Calculate elapsed time based on ticks (approximately 10 ticks per second)
                            let ticks_since_event =
                                if let Some(start_tick) = self.last_rate_limit_tick() {
                                    self.tick.saturating_sub(start_tick)
                                } else {
                                    0
                                };
                            let elapsed_secs = (ticks_since_event / 10) as u64; // ~10 ticks per second

                            self.task_fetch_info = TaskFetchInfo {
                                backoff_duration_secs: backoff_secs,
                                time_since_last_fetch_secs: elapsed_secs,
                                can_fetch_now: elapsed_secs >= backoff_secs,
                            };
                            return;
                        }
                    }
                }
            }
        }

        // No recent rate limiting, assume we can fetch
        self.task_fetch_info = TaskFetchInfo {
            backoff_duration_secs: 0,
            time_since_last_fetch_secs: 0,
            can_fetch_now: true,
        };
    }

    /// Update zkVM metrics from recent events.
    fn update_zkvm_metrics(&mut self) {
        let mut tasks_fetched = 0;
        let mut tasks_submitted = 0;
        let mut total_runtime = 0;
        let mut last_duration = 0.0;
        let mut last_status = "None".to_string();

        // Clone events to avoid borrowing issues
        let events = self.events.clone();

        // Track the full task pipeline: fetch -> prove -> submit
        for event in &events {
            match event.worker {
                Worker::TaskFetcher => {
                    // Count successful task fetches (but not rate limit responses)
                    if matches!(event.event_type, EventType::Success)
                        && !event.msg.contains("rate limited")
                        && !event.msg.contains("retrying")
                        && !event.msg.contains("Step 1 of 4")
                    {
                        tasks_fetched += 1;
                    }
                }
                Worker::Prover(_) => {
                    if matches!(event.event_type, EventType::Success) {
                        last_status = "Proved".to_string();
                        // Extract duration if present in message
                        if let Some(duration_str) = event.msg.split("took ").nth(1) {
                            if let Some(duration_num) = duration_str.split('s').next() {
                                if let Ok(duration) = duration_num.parse::<f64>() {
                                    last_duration = duration;
                                    total_runtime += duration as u64;
                                }
                            }
                        }
                    } else if matches!(event.event_type, EventType::Error) {
                        last_status = "Proof Failed".to_string();
                    }
                }
                Worker::ProofSubmitter => {
                    if matches!(event.event_type, EventType::Success)
                        && event.msg.contains("Submitted!")
                    {
                        tasks_submitted += 1;
                        last_status = "Success".to_string();
                        // Track the timestamp of last successful submission
                        self.set_last_submission_timestamp(Some(event.timestamp.clone()));
                    } else if matches!(event.event_type, EventType::Error) {
                        last_status = "Submit Failed".to_string();
                    }
                }
                _ => {}
            }
        }

        // Calculate total points: 300 points per successful submission
        let total_points = (tasks_submitted as u64) * 300;

        self.zkvm_metrics = ZkVMMetrics {
            tasks_executed: tasks_submitted.max(tasks_fetched), // Total tasks attempted
            tasks_proved: tasks_submitted,                      // Successfully completed tasks
            zkvm_runtime_secs: total_runtime,
            last_task_duration: last_duration,
            last_task_status: last_status,
            total_points,
        };
    }

    /// Update current task from recent events.
    fn update_current_task(&mut self) {
        // Look for the most recent task ID from proving events
        for event in self.events.iter().rev().take(20) {
            match event.worker {
                Worker::Prover(_) | Worker::TaskFetcher => {
                    if let Some(task_id) = extract_task_id_from_message(&event.msg) {
                        self.current_task = Some(task_id);
                        return;
                    }
                }
                _ => {}
            }
        }

        // No recent task found, clear current task
        self.current_task = None;
    }

    /// Update version information from recent events.
    fn update_version_info(&mut self) {
        // Look for the most recent version checker event
        for event in self.events.iter().rev().take(10) {
            if matches!(event.worker, Worker::VersionChecker) {
                // Check if it's an update available message
                if event.msg.contains("New version") || event.msg.contains("available") {
                    self.update_available = true;

                    // Try to extract version from message
                    if let Some(version_start) = event.msg.find("version ") {
                        let version_part = &event.msg[version_start + 8..];
                        if let Some(version_end) = version_part.find(' ') {
                            self.latest_version = Some(version_part[..version_end].to_string());
                        }
                    }
                    return;
                } else if event.msg.contains("up to date") {
                    self.update_available = false;
                    self.latest_version = None;
                    return;
                }
            }
        }
    }

    /// Update fetching state based on recent events
    fn update_fetching_state(&mut self) {
        let now = Instant::now();

        // Check for completion or error to reset to idle first
        for event in self.events.iter().rev().take(5) {
            if matches!(event.worker, Worker::TaskFetcher)
                && matches!(event.event_type, EventType::Success | EventType::Error)
                && !event.msg.contains("Step 1 of 4")
            {
                self.set_fetching_state(FetchingState::Idle);
                return;
            }
        }

        // Check for fetching activity in recent events ONLY if not already active
        if !matches!(self.fetching_state(), FetchingState::Active { .. }) {
            for event in self.events.iter().rev().take(10) {
                if matches!(event.worker, Worker::TaskFetcher)
                    && event.msg.contains("Step 1 of 4: Requesting task...")
                {
                    // Start fetching state ONLY if not already active
                    self.set_fetching_state(FetchingState::Active { started_at: now });
                    return;
                }
            }
        }

        // Check for timeout (5 seconds max) if currently active
        if let FetchingState::Active { started_at } = self.fetching_state() {
            if started_at.elapsed().as_secs() > 5 {
                self.set_fetching_state(FetchingState::Timeout);
            }
        }
    }

    /// Update proving state based on recent events
    fn update_proving_state(&mut self) {
        let now = Instant::now();

        // Check for proving activity in recent events
        for event in self.events.iter().rev().take(10) {
            if let Worker::Prover(_) = event.worker {
                if event.msg.contains("Step 2 of 4: Computing") {
                    // Extract task ID if available
                    let task_id = event
                        .msg
                        .split("Task-")
                        .nth(1)
                        .and_then(|s| s.split_whitespace().next())
                        .map(|s| format!("Task-{}", s));

                    self.set_proving_state(ProvingState::Active {
                        started_at: now,
                        task_id,
                    });
                    return;
                }

                // Check for completion to reset to idle
                if matches!(event.event_type, EventType::Success | EventType::Error)
                    && !event.msg.contains("Step 2 of 4")
                {
                    self.set_proving_state(ProvingState::Idle);
                    return;
                }
            }
        }
    }

    /// Update current prover state from state change events
    fn update_prover_state(&mut self) {
        // Look for the most recent state change event
        for event in self.events.iter().rev().take(10) {
            if event.event_type == EventType::StateChange {
                if let Some(state) = event.prover_state {
                    self.set_current_prover_state(state);
                    return;
                }
            }
        }
    }
}
