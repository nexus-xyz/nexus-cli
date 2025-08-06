//! Dashboard utility functions
//!
//! Contains helper functions used across dashboard components

use crate::events::{Event as WorkerEvent, EventType, Worker};
use ratatui::prelude::Color;
use std::collections::VecDeque;

/// Get a ratatui color for a worker based on its type
pub fn get_worker_color(worker: &Worker) -> Color {
    match worker {
        Worker::TaskFetcher => Color::Cyan,
        Worker::Prover(_) => Color::Yellow,
        Worker::ProofSubmitter => Color::Green,
        Worker::VersionChecker => Color::Magenta,
    }
}

/// Format compact timestamp with date and time from full timestamp
pub fn format_compact_timestamp(timestamp: &str) -> String {
    // Extract from "YYYY-MM-DD HH:MM:SS" format
    if let Some(date_part) = timestamp.split(' ').next() {
        if let Some(time_part) = timestamp.split(' ').nth(1) {
            // Extract MM-DD from date and HH:MM from time
            if let Some(month_day) = date_part.get(5..10) {
                // Get MM-DD
                if let Some(hour_min) = time_part.get(0..5) {
                    // Get HH:MM
                    return format!("{} {}", month_day, hour_min);
                }
            }
        }
    }
    // Fallback to original timestamp if parsing fails
    timestamp.to_string()
}

/// Clean HTTP error messages
pub fn clean_http_error_message(msg: &str) -> String {
    // Replace verbose HTTP error patterns with cleaner messages
    if msg.contains("reqwest::Error") && msg.contains("ConnectTimeout") {
        return "Connection timeout - retrying...".to_string();
    }
    if msg.contains("reqwest::Error") && msg.contains("TimedOut") {
        return "Request timed out - retrying...".to_string();
    }
    if msg.contains("reqwest::Error") {
        return "Network error - retrying...".to_string();
    }
    // Return original message if no HTTP error pattern detected
    msg.to_string()
}

/// Extract task ID from an event message.
pub fn extract_task_id_from_message(msg: &str) -> Option<String> {
    if let Some(task_start) = msg.find("Task-") {
        // Find the end of the task ID (space, newline, or end of string)
        let remaining = &msg[task_start..];
        if let Some(task_end) = remaining.find(|c: char| c.is_whitespace() || c == '\n') {
            return Some(remaining[..task_end].to_string());
        } else if remaining.len() > 5 {
            // "Task-" prefix is 5 chars
            return Some(remaining.to_string());
        }
    }
    None
}

/// Get elapsed time for current prover state from events
pub fn get_current_state_elapsed_secs(
    events: &VecDeque<WorkerEvent>,
    current_prover_state: crate::events::ProverState,
) -> u64 {
    // Look for the most recent state change event with timer
    for event in events.iter().rev().take(10) {
        if event.event_type == EventType::StateChange {
            if let Some(prover_state) = event.prover_state {
                if prover_state == current_prover_state {
                    if let Some(start_time) = event.state_start_time {
                        return start_time.elapsed().as_secs();
                    }
                }
            }
        }
    }
    0
}
