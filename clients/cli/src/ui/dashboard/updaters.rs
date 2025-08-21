//! Dashboard state update logic
//!
//! Contains all methods for updating dashboard state from events

use super::state::{DashboardState, FetchingState};

use crate::events::{Event as WorkerEvent, EventType, Worker};
use crate::ui::metrics::{SystemMetrics, TaskFetchInfo};

use std::time::Instant;

impl DashboardState {
    /// Update the dashboard state with new tick and metrics.
    pub fn update(&mut self) {
        self.tick += 1;

        // Update system metrics using persistent sysinfo instance for accurate CPU measurements
        let previous_peak = self.system_metrics.peak_ram_bytes;
        let previous_metrics = self.system_metrics.clone();
        self.system_metrics = SystemMetrics::update(
            self.get_sysinfo_mut(),
            previous_peak,
            Some(&previous_metrics),
        );

        // Process all queued events one by one
        while let Some(event) = self.pending_events.pop_front() {
            // Add to activity logs for display
            self.add_to_activity_log(event.clone());

            // Process the event for state updates
            self.process_event(&event);
        }

        // Handle timeout logic (doesn't need events)
        self.check_fetching_timeout();

        // Update task fetch info based on current state
        self.update_task_fetch_countdown();
    }

    /// Process a single event and update relevant state
    fn process_event(&mut self, event: &WorkerEvent) {
        match event.worker {
            Worker::TaskFetcher => self.handle_task_fetcher_event(event),
            Worker::Prover(_) => self.handle_prover_event(event),
            Worker::ProofSubmitter => self.handle_proof_submitter_event(event),
        }

        // Handle state changes regardless of worker
        if event.event_type == EventType::StateChange {
            if let Some(state) = event.prover_state {
                self.set_current_prover_state(state);
            }
        }
    }

    /// Handle TaskFetcher events
    fn handle_task_fetcher_event(&mut self, event: &WorkerEvent) {
        // Handle task ID extraction from "Got task" success events
        if matches!(event.event_type, EventType::Success)
            && event.msg.contains("Step 1 of 4: Got task")
        {
            if let Some(task_id) = Self::extract_task_id(&event.msg) {
                self.last_task = self.current_task.clone();
                self.current_task = Some(task_id);

                // Count this as a task fetch if we haven't seen this task before
                self.zkvm_metrics.tasks_fetched += 1;
                // Track Step 2 start (proving begins at the end of Step 1)
                self.step2_start_time = Some(Instant::now());
            }
        }

        // Handle fetching state changes
        if Self::is_completion_event(event) {
            self.set_fetching_state(FetchingState::Idle);
        } else if Self::is_fetching_start_event(event)
            && !matches!(self.fetching_state(), FetchingState::Active { .. })
        {
            self.set_fetching_state(FetchingState::Active {
                started_at: Instant::now(),
            });
        }

        // Handle waiting messages for task fetch info
        if event.msg.contains("ready for next task") {
            if let Some(seconds) = Self::extract_wait_seconds(&event.msg) {
                let is_same_message = match &self.waiting_start_info {
                    Some((_, prev_wait)) => *prev_wait == seconds,
                    None => false,
                };

                if !is_same_message {
                    self.waiting_start_info = Some((Instant::now(), seconds));
                }
            }
        }
    }

    /// Handle Prover events
    fn handle_prover_event(&mut self, event: &WorkerEvent) {
        if matches!(event.event_type, EventType::Success) {
            // Track Step 3 completion (proof generated)
            if event.msg.contains("Step 3 of 4: Proof generated for task") {
                if let Some(start_time) = self.step2_start_time {
                    self.zkvm_metrics.zkvm_runtime_secs += start_time.elapsed().as_secs();
                    self.zkvm_metrics.last_task_status = "Proved".to_string();
                    self.step2_start_time = None;
                }
            }
        } else if matches!(event.event_type, EventType::Error) {
            self.zkvm_metrics.last_task_status = "Proof Failed".to_string();
            self.step2_start_time = None; // Clear timing for failed proof
        }
    }

    /// Handle ProofSubmitter events
    fn handle_proof_submitter_event(&mut self, event: &WorkerEvent) {
        if matches!(event.event_type, EventType::Success)
            && event
                .msg
                .contains("Step 4 of 4: Proof submitted successfully")
        {
            // If we see a Step 4 completion but have fewer fetched tasks,
            // it means we missed earlier events (dashboard started after task began)
            self.zkvm_metrics.tasks_submitted += 1;
            self.zkvm_metrics.tasks_fetched = self
                .zkvm_metrics
                .tasks_fetched
                .max(self.zkvm_metrics.tasks_submitted);

            self.zkvm_metrics.last_task_status = "Success".to_string();
            self.set_last_submission_timestamp(Some(event.timestamp.clone()));

            // Update total points
            self.zkvm_metrics._total_points = (self.zkvm_metrics.tasks_submitted as u64) * 300;
        } else if matches!(event.event_type, EventType::Error) {
            self.zkvm_metrics.last_task_status = "Submit Failed".to_string();
        }
    }

    /// Update task fetch countdown based on current waiting state
    fn update_task_fetch_countdown(&mut self) {
        if let Some((start_time, original_secs)) = &self.waiting_start_info {
            let elapsed_secs = start_time.elapsed().as_secs();
            let remaining_secs = original_secs.saturating_sub(elapsed_secs);

            self.task_fetch_info = TaskFetchInfo {
                backoff_duration_secs: *original_secs,
                time_since_last_fetch_secs: elapsed_secs,
                can_fetch_now: remaining_secs == 0,
            };

            // Clear expired countdown
            if remaining_secs == 0 {
                self.waiting_start_info = None;
            }
        } else {
            // No active countdown, assume we can fetch
            self.task_fetch_info = TaskFetchInfo {
                backoff_duration_secs: 0,
                time_since_last_fetch_secs: 0,
                can_fetch_now: true,
            };
        }
    }

    /// Check for fetching timeout (doesn't need events)
    fn check_fetching_timeout(&mut self) {
        if let FetchingState::Active { started_at } = self.fetching_state() {
            if started_at.elapsed().as_secs() > 5 {
                self.set_fetching_state(FetchingState::Timeout);
            }
        }
    }
}

// Helper functions for event parsing
impl DashboardState {
    /// Extract task ID from message. Expected format: "Step 1 of 4: Got task TASK_ID"
    fn extract_task_id(msg: &str) -> Option<String> {
        let pattern = "Got task ";
        let start = msg.find(pattern)?;
        let task_id_start = start + pattern.len();
        let remaining = &msg[task_id_start..];

        // Find end of task ID (space, newline, or end of string)
        if let Some(end) = remaining.find(|c: char| c.is_whitespace() || c == '\n') {
            Some(remaining[..end].to_string())
        } else if !remaining.is_empty() {
            Some(remaining.to_string())
        } else {
            None
        }
    }

    /// Extract wait seconds from message. Expected format: "...ready for next task (30) seconds"
    fn extract_wait_seconds(msg: &str) -> Option<u64> {
        let start = msg.find("(")?;
        let end = msg[start..].find(") seconds")?;
        msg[start + 1..start + end].parse().ok()
    }

    /// Check if event indicates task completion or error (not Step 1)
    fn is_completion_event(event: &WorkerEvent) -> bool {
        matches!(event.worker, Worker::TaskFetcher)
            && matches!(event.event_type, EventType::Success | EventType::Error)
            && !event.msg.contains("Step 1 of 4")
    }

    /// Check if event indicates fetching activity start
    fn is_fetching_start_event(event: &WorkerEvent) -> bool {
        matches!(event.worker, Worker::TaskFetcher)
            && event.msg.contains("Step 1 of 4: Requesting task...")
    }
}
