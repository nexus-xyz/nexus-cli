//! Dashboard state update logic
//!
//! Contains all methods for updating dashboard state from events

use super::state::{DashboardState, FetchingState};

use crate::events::{EventType, Worker};
use crate::ui::metrics::{SystemMetrics, TaskFetchInfo, ZkVMMetrics};

use std::time::Instant;

impl DashboardState {
    /// Update the dashboard state with new tick and metrics.
    pub fn update(&mut self) {
        self.tick += 1;

        // Update current task from recent events
        self.update_current_task();

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

        // Update fetching state
        self.update_fetching_state();

        // Update current prover state from state events
        self.update_prover_state();
    }

    /// Update task fetch info from recent events.
    /// We have to do string search here because we have to support both headless and TUI modes,
    /// thus string search is the unified approach between the two.
    /// We can improve this by using a more efficient approach, but this is a good starting point.
    /// TODO: Improve this by using a more efficient approach.
    fn update_task_fetch_info(&mut self) {
        // Look for the most recent waiting message (expanded search window)
        for event in self.events.iter().rev().take(20) {
            if matches!(event.worker, Worker::TaskFetcher) {
                // Only process "ready for next task" messages
                if event.msg.contains("ready for next task") {
                    // More robust parsing: look specifically for the pattern "(number) seconds"
                    if let Some(seconds) = self.extract_wait_seconds(&event.msg) {
                        // Check if this is the EXACT SAME waiting message we've seen before
                        let is_same_message = match &self.waiting_start_info {
                            Some((_, prev_wait)) => *prev_wait == seconds,
                            None => false,
                        };

                        if !is_same_message {
                            // This is a NEW waiting period - reset tracking
                            self.waiting_start_info = Some((Instant::now(), seconds));
                        }

                        // Calculate elapsed time since we started tracking this specific wait period
                        if let Some((start_time, original_secs)) = &self.waiting_start_info {
                            let elapsed_secs = start_time.elapsed().as_secs();
                            let remaining_secs = original_secs.saturating_sub(elapsed_secs);

                            self.task_fetch_info = TaskFetchInfo {
                                backoff_duration_secs: *original_secs,
                                time_since_last_fetch_secs: elapsed_secs,
                                can_fetch_now: remaining_secs == 0,
                            };
                            return;
                        }
                    }
                }
            }
        }

        // No recent waiting message found - preserve existing countdown if still valid
        if let Some((start_time, original_secs)) = &self.waiting_start_info {
            let elapsed_secs = start_time.elapsed().as_secs();
            let remaining_secs = original_secs.saturating_sub(elapsed_secs);

            // If we still have time remaining on the existing countdown, preserve it
            if remaining_secs > 0 {
                self.task_fetch_info = TaskFetchInfo {
                    backoff_duration_secs: *original_secs,
                    time_since_last_fetch_secs: elapsed_secs,
                    can_fetch_now: false,
                };
                return;
            } else {
                // Countdown has expired, clear the waiting info
                self.waiting_start_info = None;
            }
        }

        // No active countdown, assume we can fetch
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
        let mut last_status = "None".to_string();

        // Clone events to avoid borrowing issues
        let events = self.events.clone();

        // Process events to update timings and counts
        for event in &events {
            match event.worker {
                Worker::TaskFetcher => {
                    // Count successful task fetches (but not rate limit responses)
                    if matches!(event.event_type, EventType::Success)
                        && event.msg.contains("Step 1 of 4: Got task")
                    {
                        tasks_fetched += 1;
                    }
                }
                Worker::Prover(_) => {
                    if matches!(event.event_type, EventType::Success) {
                        // Track Step 2 start (proving begins)
                        if event.msg.contains("Step 2 of 4: Proving task") {
                            self.step2_start_time = Some(Instant::now());
                        }
                        // Track Step 3 completion (proof generated) - accumulate runtime
                        else if event.msg.contains("Step 3 of 4: Proof generated for task") {
                            if let Some(start_time) = self.step2_start_time.take() {
                                let duration = start_time.elapsed();
                                let duration_secs = duration.as_secs_f64();
                                if duration_secs > 0.0 {
                                    self.accumulated_runtime_secs += duration_secs as u64;
                                    last_status = "Proved".to_string();
                                }
                            }
                        }
                    } else if matches!(event.event_type, EventType::Error) {
                        last_status = "Proof Failed".to_string();
                    }
                }
                Worker::ProofSubmitter => {
                    if matches!(event.event_type, EventType::Success)
                        && event
                            .msg
                            .contains("Step 4 of 4: Proof submitted successfully")
                    {
                        tasks_submitted += 1;
                        last_status = "Success".to_string();
                        // Track the timestamp of last successful submission
                        self.set_last_submission_timestamp(Some(event.timestamp.clone()));
                    } else if matches!(event.event_type, EventType::Error) {
                        last_status = "Submit Failed".to_string();
                    }
                }
            }
        }

        // Calculate total points: 300 points per successful submission
        // TODO: Add estimated points for successful proofs or fetch points from server.
        let _total_points = (tasks_submitted as u64) * 300;

        self.zkvm_metrics = ZkVMMetrics {
            tasks_executed: tasks_fetched,                    // Total tasks fetched
            tasks_proved: tasks_submitted,                    // Successfully completed tasks
            zkvm_runtime_secs: self.accumulated_runtime_secs, // Use accumulated runtime across all tasks
            last_task_status: last_status,
            _total_points,
        };
    }

    /// Update current task from recent events.
    fn update_current_task(&mut self) {
        // Look for the most recent task ID from proving events
        for event in self.events.iter().rev().take(20) {
            match event.worker {
                Worker::Prover(_) | Worker::TaskFetcher => {
                    // Extract task ID inline
                    if let Some(task_start) = event.msg.find("Task-") {
                        // Find the end of the task ID (space, newline, or end of string)
                        let remaining = &event.msg[task_start..];
                        if let Some(task_end) =
                            remaining.find(|c: char| c.is_whitespace() || c == '\n')
                        {
                            self.current_task = Some(remaining[..task_end].to_string());
                            return;
                        } else if remaining.len() > 5 {
                            // "Task-" prefix is 5 chars
                            self.current_task = Some(remaining.to_string());
                            return;
                        }
                    }
                }
                _ => {}
            }
        }

        // No recent task found, clear current task
        self.current_task = None;
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

    /// Extract wait seconds from log message using robust parsing.
    /// Expected format: "Step 1 of 4: Waiting - ready for next task (30) seconds"
    fn extract_wait_seconds(&self, msg: &str) -> Option<u64> {
        // Look for pattern: (number) seconds
        // Find the last opening parenthesis before "seconds"
        if let Some(seconds_pos) = msg.find(" seconds") {
            let before_seconds = &msg[..seconds_pos];

            // Find the last opening parenthesis
            if let Some(open_pos) = before_seconds.rfind('(') {
                // Find the closing parenthesis after the opening one
                let after_open = &msg[open_pos + 1..];
                if let Some(close_pos) = after_open.find(')') {
                    let number_str = &after_open[..close_pos];

                    // Parse the number, trimming whitespace
                    match number_str.trim().parse::<u64>() {
                        Ok(seconds) => {
                            // Debug: Successful parsing
                            #[cfg(debug_assertions)]
                            eprintln!(
                                "[DEBUG] Successfully parsed wait time: {} seconds from: {}",
                                seconds, msg
                            );
                            return Some(seconds);
                        }
                        Err(_) => {
                            // Debug: Failed to parse number
                            #[cfg(debug_assertions)]
                            eprintln!(
                                "[DEBUG] Failed to parse number '{}' from message: {}",
                                number_str.trim(),
                                msg
                            );
                        }
                    }
                } else {
                    // Debug: No closing parenthesis found
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "[DEBUG] No closing parenthesis found after position {} in: {}",
                        open_pos, msg
                    );
                }
            } else {
                // Debug: No opening parenthesis found
                #[cfg(debug_assertions)]
                eprintln!(
                    "[DEBUG] No opening parenthesis found before 'seconds' in: {}",
                    msg
                );
            }
        } else {
            // Debug: Pattern " seconds" not found
            #[cfg(debug_assertions)]
            eprintln!("[DEBUG] Pattern ' seconds' not found in: {}", msg);
        }

        None
    }
}
