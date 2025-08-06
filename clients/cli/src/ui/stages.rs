//! Enhanced prover stage management with realistic transitions and timing.

use crate::events::{Event as WorkerEvent, EventType, Worker};
use crate::ui::metrics::TaskFetchInfo;
use std::collections::VecDeque;
use std::time::Instant;

/// Represents the different stages of the proving process with enhanced timing.
#[derive(Clone, Debug)]
pub enum ProverStage {
    /// No active task - waiting for user or system to start.
    Idle,
    /// Waiting to fetch a new task with real countdown timer from TaskFetchState.
    WaitingToFetch {
        remaining_secs: u64,
        total_backoff_secs: u64,
    },
    /// Fetching a task from the orchestrator.
    Fetching {
        elapsed_secs: u32,
        estimated_total: u32,
        started_at: Instant,
    },
    /// Currently proving a task with animated progress.
    Proving {
        task_id: Option<String>,
        elapsed_secs: u32,
        animation_frame: usize,
        started_at: Instant,
    },
    /// Submitting proof to the orchestrator.
    Submitting {
        elapsed_secs: u32,
        estimated_total: u32,
        started_at: Instant,
        proof_size_mb: Option<f32>,
    },
    /// Task completed successfully.
    Completed {
        task_id: Option<String>,
        completion_time: Instant,
        points_earned: Option<u32>,
    },
}

impl Default for ProverStage {
    fn default() -> Self {
        Self::Idle
    }
}

impl ProverStage {
    /// Update the stage based on recent events, TaskFetchInfo, and elapsed time.
    pub fn update_from_events(
        events: &VecDeque<WorkerEvent>,
        current_stage: &ProverStage,
        tick: usize,
        fetch_info: &TaskFetchInfo,
    ) -> Self {
        // Get the most recent events to determine current activity
        let recent_events: Vec<_> = events.iter().rev().take(5).collect();

        // Check for completion and transition to countdown
        if let Some(_completion_event) = recent_events.iter().find(|e| {
            matches!(e.worker, Worker::ProofSubmitter)
                && matches!(e.event_type, EventType::Success)
                && e.msg.contains("Submitted")
        }) {
            // If we just completed and can't fetch now, show waiting state
            if !fetch_info.can_fetch_now {
                let remaining = fetch_info
                    .backoff_duration_secs
                    .saturating_sub(fetch_info.time_since_last_fetch_secs);

                return Self::WaitingToFetch {
                    remaining_secs: remaining,
                    total_backoff_secs: fetch_info.backoff_duration_secs,
                };
            }
        }

        // Handle countdown timer logic based on real TaskFetchState
        if !fetch_info.can_fetch_now {
            let remaining = fetch_info
                .backoff_duration_secs
                .saturating_sub(fetch_info.time_since_last_fetch_secs);

            return Self::WaitingToFetch {
                remaining_secs: remaining,
                total_backoff_secs: fetch_info.backoff_duration_secs,
            };
        }

        // Check for active fetching
        if let Some(_fetch_event) = recent_events.iter().find(|e| {
            matches!(e.worker, Worker::TaskFetcher)
                && matches!(
                    e.event_type,
                    EventType::Waiting | EventType::Success | EventType::Refresh
                )
        }) {
            match current_stage {
                ProverStage::Fetching {
                    started_at,
                    estimated_total,
                    ..
                } => {
                    let elapsed = started_at.elapsed().as_secs() as u32;
                    if elapsed >= *estimated_total {
                        // Transition to proving
                        Self::Proving {
                            task_id: Self::extract_task_id(&recent_events),
                            elapsed_secs: 0,
                            animation_frame: tick % 60,
                            started_at: Instant::now(),
                        }
                    } else {
                        Self::Fetching {
                            elapsed_secs: elapsed,
                            estimated_total: *estimated_total,
                            started_at: *started_at,
                        }
                    }
                }
                _ => Self::Fetching {
                    elapsed_secs: 0,
                    estimated_total: 15,
                    started_at: Instant::now(),
                },
            }
        }
        // Check for active proving
        else if let Some(prove_event) = recent_events.iter().find(|e| {
            matches!(e.worker, Worker::Prover(_)) && matches!(e.event_type, EventType::Success)
        }) {
            match current_stage {
                ProverStage::Proving {
                    started_at,
                    task_id,
                    ..
                } => {
                    let elapsed = started_at.elapsed().as_secs() as u32;
                    // After some time proving, transition to submitting
                    if elapsed >= 30 || prove_event.msg.contains("Computing") {
                        Self::Submitting {
                            elapsed_secs: 0,
                            estimated_total: 10,
                            started_at: Instant::now(),
                            proof_size_mb: Some(2.4), // Simulated proof size
                        }
                    } else {
                        Self::Proving {
                            task_id: task_id.clone(),
                            elapsed_secs: elapsed,
                            animation_frame: tick % 60,
                            started_at: *started_at,
                        }
                    }
                }
                _ => Self::Proving {
                    task_id: Self::extract_task_id(&recent_events),
                    elapsed_secs: 0,
                    animation_frame: tick % 60,
                    started_at: Instant::now(),
                },
            }
        }
        // Check for active submitting
        else if let Some(_submit_event) = recent_events.iter().find(|e| {
            matches!(e.worker, Worker::ProofSubmitter) && matches!(e.event_type, EventType::Waiting)
        }) {
            match current_stage {
                ProverStage::Submitting {
                    started_at,
                    estimated_total,
                    proof_size_mb,
                    ..
                } => {
                    let elapsed = started_at.elapsed().as_secs() as u32;
                    Self::Submitting {
                        elapsed_secs: elapsed,
                        estimated_total: *estimated_total,
                        started_at: *started_at,
                        proof_size_mb: *proof_size_mb,
                    }
                }
                _ => Self::Submitting {
                    elapsed_secs: 0,
                    estimated_total: 10,
                    started_at: Instant::now(),
                    proof_size_mb: Some(2.4),
                },
            }
        } else {
            // No recent activity, maintain current stage or go idle
            match current_stage {
                ProverStage::WaitingToFetch { .. } => current_stage.clone(),
                ProverStage::Fetching { .. } => current_stage.clone(),
                ProverStage::Proving { .. } => current_stage.clone(),
                ProverStage::Submitting { .. } => current_stage.clone(),
                _ => Self::Idle,
            }
        }
    }

    /// Extract task ID from recent events.
    fn extract_task_id(events: &[&WorkerEvent]) -> Option<String> {
        for event in events {
            if let Some(task_start) = event.msg.find("Task-") {
                if let Some(task_end) = event.msg[task_start..].find(' ') {
                    return Some(event.msg[task_start..task_start + task_end].to_string());
                }
            }
        }
        None
    }

    /// Get the progress ratio (0.0 to 1.0) for the current stage.
    pub fn progress_ratio(&self) -> f64 {
        match self {
            Self::Idle => 0.0,
            Self::WaitingToFetch {
                remaining_secs,
                total_backoff_secs,
                ..
            } => {
                if *total_backoff_secs == 0 {
                    1.0
                } else {
                    let elapsed = total_backoff_secs - remaining_secs;
                    (elapsed as f64) / (*total_backoff_secs as f64)
                }
            }
            Self::Fetching {
                elapsed_secs,
                estimated_total,
                ..
            } => (*elapsed_secs as f64 / *estimated_total as f64).min(1.0),
            Self::Proving {
                animation_frame, ..
            } => {
                // Create a smooth sine wave animation
                let cycle = *animation_frame as f64 / 60.0 * 2.0 * std::f64::consts::PI;
                (cycle.sin() + 1.0) / 2.0 // Normalize to 0.0-1.0
            }
            Self::Submitting {
                elapsed_secs,
                estimated_total,
                ..
            } => (*elapsed_secs as f64 / *estimated_total as f64).min(1.0),
            Self::Completed { .. } => 1.0,
        }
    }

    /// Get the display text for the current stage.
    pub fn display_text(&self) -> String {
        match self {
            Self::Idle => "IDLE - Ready to start proving".to_string(),
            Self::WaitingToFetch { remaining_secs, .. } => {
                format!("WAITING - Next task in {}s", remaining_secs)
            }
            Self::Fetching {
                elapsed_secs,
                estimated_total,
                ..
            } => {
                format!(
                    "FETCHING - Getting next task ({}/{}s)",
                    elapsed_secs, estimated_total
                )
            }
            Self::Proving {
                task_id,
                elapsed_secs,
                ..
            } => {
                if let Some(id) = task_id {
                    format!("PROVING - {} ({}s)", id, elapsed_secs)
                } else {
                    format!("PROVING - Generating proof ({}s)", elapsed_secs)
                }
            }
            Self::Submitting {
                elapsed_secs,
                estimated_total,
                proof_size_mb,
                ..
            } => {
                if let Some(size) = proof_size_mb {
                    format!(
                        "SUBMITTING - {:.1}MB proof ({}/{}s)",
                        size, elapsed_secs, estimated_total
                    )
                } else {
                    format!(
                        "SUBMITTING - Uploading proof ({}/{}s)",
                        elapsed_secs, estimated_total
                    )
                }
            }
            Self::Completed { points_earned, .. } => {
                if let Some(points) = points_earned {
                    format!("COMPLETED - Earned {} points!", points)
                } else {
                    "COMPLETED - Task finished successfully!".to_string()
                }
            }
        }
    }

    /// Get the color for the current stage.
    pub fn color(&self) -> ratatui::prelude::Color {
        use ratatui::prelude::Color;
        match self {
            Self::Idle => Color::DarkGray,
            Self::WaitingToFetch { .. } => Color::LightBlue,
            Self::Fetching { .. } => Color::Cyan,
            Self::Proving { .. } => Color::Yellow,
            Self::Submitting { .. } => Color::LightGreen,
            Self::Completed { .. } => Color::Green,
        }
    }

    /// Get status emoji for the current stage.
    pub fn status_text(&self) -> &'static str {
        match self {
            Self::Idle => "😴",
            Self::WaitingToFetch { .. } => "⏰",
            Self::Fetching { .. } => "🔍",
            Self::Proving { .. } => "⚡",
            Self::Submitting { .. } => "📤",
            Self::Completed { .. } => "🎉",
        }
    }
}
