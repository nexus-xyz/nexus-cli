//! Dashboard utility functions
//!
//! Contains helper functions used across dashboard components

use crate::events::Worker;
use ratatui::prelude::Color;

/// Get a ratatui color for a worker based on its type
pub fn get_worker_color(worker: &Worker) -> Color {
    match worker {
        Worker::TaskFetcher => Color::Cyan,
        Worker::Prover(_) => Color::Yellow,
        Worker::ProofSubmitter => Color::Green,
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
