//! Dashboard screen rendering.

use crate::environment::Environment;
use crate::events::{Event as WorkerEvent, EventType, Worker};
use crate::system;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::collections::VecDeque;
use std::time::Instant;

/// State for the dashboard screen, containing node information and menu items.
#[derive(Debug, Clone)]
pub struct DashboardState {
    /// Unique identifier for the node.
    pub node_id: Option<u64>,

    /// The environment in which the application is running.
    pub environment: Environment,

    /// Total NEX points available to the node, if any.
    pub nex_points: Option<u64>,

    /// The start time of the application, used for computing uptime.
    pub start_time: Instant,

    /// The current task being executed by the node, if any.
    pub current_task: Option<String>,

    /// Total number of (virtual) CPU cores available on the machine.
    pub total_cores: usize,

    /// Total RAM available on the machine, in GB.
    pub total_ram_gb: f64,

    /// A queue of events received from worker threads.
    pub events: VecDeque<WorkerEvent>,

    /// Whether a new version is available.
    pub update_available: bool,

    /// The latest version string, if known.
    pub latest_version: Option<String>,

    /// Whether to disable background colors
    pub no_background_color: bool,
}

impl DashboardState {
    /// Creates a new instance of the dashboard state.
    ///
    /// # Arguments
    /// * `node_id` - This node's unique identifier, if available.
    /// * `start_time` - The start time of the application, used for computing uptime.
    /// * `environment` - The environment in which the application is running.
    /// * `no_background_color` - Whether to disable background colors
    pub fn new(
        node_id: Option<u64>,
        environment: Environment,
        start_time: Instant,
        events: &VecDeque<WorkerEvent>,
        no_background_color: bool,
    ) -> Self {
        // Check for version update messages in recent events
        let (update_available, latest_version, _) = Self::check_for_version_updates(events);

        Self {
            node_id,
            environment,
            nex_points: None,
            start_time,
            current_task: None,
            total_cores: system::num_cores(),
            total_ram_gb: system::total_memory_gb(),
            events: events.clone(),
            update_available,
            latest_version,
            no_background_color,
        }
    }

    /// Check recent events for version update information
    fn check_for_version_updates(
        events: &VecDeque<WorkerEvent>,
    ) -> (
        bool,
        Option<String>,
        Option<crate::version_requirements::ConstraintType>,
    ) {
        // Look for the most recent version checker event
        for event in events.iter().rev() {
            if matches!(event.worker, Worker::VersionChecker) {
                // Show all version checker events (not just success events)
                // This includes blocking, warning, and notice constraints
                return (true, None, None);
            }
        }

        (false, None, None)
    }

    /// Get a ratatui color for a worker based on its type and ID
    fn get_worker_color(worker: &Worker) -> Color {
        match worker {
            Worker::TaskFetcher => Color::Cyan,
            Worker::Prover(worker_id) => {
                // Cycle through different colors for different worker IDs
                let colors = [
                    Color::Green,
                    Color::Yellow,
                    Color::Magenta,
                    Color::Blue,
                    Color::Red,
                    Color::LightGreen,
                    Color::LightYellow,
                    Color::LightMagenta,
                    Color::LightBlue,
                    Color::LightRed,
                ];
                colors[*worker_id % colors.len()]
            }
            Worker::ProofSubmitter => Color::White,
            Worker::VersionChecker => Color::LightCyan,
        }
    }

    /// Format timestamp to include date but no year (MM-DD HH:MM:SS)
    fn format_compact_timestamp(timestamp: &str) -> String {
        // Extract from "YYYY-MM-DD HH:MM:SS" format to "MM-DD HH:MM:SS"
        if let Some(date_time) = timestamp.split_once(' ') {
            let date_part = date_time.0; // "YYYY-MM-DD"
            let time_part = date_time.1; // "HH:MM:SS"

            if let Some(month_day) = date_part.get(5..) {
                // Skip "YYYY-"
                format!("{} {}", month_day, time_part)
            } else {
                timestamp.to_string()
            }
        } else {
            timestamp.to_string()
        }
    }

    /// Clean HTTP error messages to show only essential information
    fn clean_http_error_message(msg: &str) -> String {
        // Handle common HTTP error patterns with HTML content
        if msg.contains("<html>") || msg.contains("<!DOCTYPE") {
            // Extract specific HTTP status codes
            if msg.contains("502") {
                return "❌ HTTP 502 Bad Gateway".to_string();
            }
            if msg.contains("503") {
                return "❌ HTTP 503 Service Unavailable".to_string();
            }
            if msg.contains("504") {
                return "❌ HTTP 504 Gateway Timeout".to_string();
            }
            if msg.contains("500") {
                return "❌ HTTP 500 Internal Server Error".to_string();
            }
            if msg.contains("429") {
                return "⏳ HTTP 429 Rate Limited".to_string();
            }
            // Generic fallback for other HTML error responses
            return "❌ HTTP Error (server returned HTML)".to_string();
        }

        // Handle messages with "status XXX:" pattern (clean format)
        if let Some(status_pos) = msg.find("status ") {
            if let Some(status_end) = msg[status_pos..]
                .find(':')
                .or_else(|| msg[status_pos..].find('<'))
            {
                let status_part = &msg[..status_pos + status_end];
                // Look for additional context before "status"
                if let Some(error_start) = status_part
                    .rfind("error")
                    .or_else(|| status_part.rfind("Error"))
                {
                    return format!("❌ {}", &status_part[error_start..]);
                } else {
                    return format!("❌ HTTP {}", &status_part[status_pos..]);
                }
            }
        }

        // Return original message if no HTTP error pattern detected
        msg.to_string()
    }
}

/// Render the dashboard screen.
pub fn render_dashboard(f: &mut Frame, state: &DashboardState) {
    // Only apply background color if no_background_color is false
    if !state.no_background_color {
        let background_block = Block::default().style(Style::default().bg(Color::Rgb(18, 18, 24)));
        f.render_widget(background_block, f.area());
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Title block
                Constraint::Min(0),    // Body area
                Constraint::Length(2), // Footer block
            ]
            .as_ref(),
        )
        .split(f.area());

    // Title section with version info
    let version = env!("CARGO_PKG_VERSION");
    let title_text = if state.update_available {
        if let Some(latest) = &state.latest_version {
            format!(
                "=== NEXUS PROVER v{} → 🚀 {} UPDATE AVAILABLE ===",
                version, latest
            )
        } else {
            format!("=== NEXUS PROVER v{} → 🚀 UPDATE AVAILABLE ===", version)
        }
    } else {
        format!("=== NEXUS PROVER v{} ===", version)
    };

    let title_color = if state.update_available {
        // Look for the most recent version checker event to determine color
        let mut version_color = Color::LightYellow; // Default fallback
        for event in state.events.iter().rev() {
            if matches!(event.worker, Worker::VersionChecker) {
                version_color = match (event.event_type, event.log_level) {
                    (EventType::Error, crate::error_classifier::LogLevel::Error) => Color::Red,
                    (EventType::Error, crate::error_classifier::LogLevel::Warn) => {
                        Color::LightYellow
                    }
                    (EventType::Success, _) => Color::Cyan,
                    _ => Color::LightYellow,
                };
                break;
            }
        }
        version_color
    } else {
        Color::Cyan
    };

    let title_block = Block::default().borders(Borders::BOTTOM);
    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        )
        .block(title_block);
    f.render_widget(title, chunks[0]);

    // Body layout: Split into two columns (status and logs)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(28), Constraint::Percentage(72)].as_ref())
        .split(chunks[1]);

    // Status Section
    let status_block = Block::default()
        .borders(Borders::RIGHT)
        .title("STATUS")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    let mut status_lines = Vec::new();

    // Display the node ID, if any, or "Not connected" if not available
    let node_id_text = if let Some(id) = state.node_id {
        format!("NODE ID: {}", id)
    } else {
        "NODE ID: Not connected".to_string()
    };
    status_lines.push(Line::from(node_id_text));

    // Environment
    status_lines.push(Line::from(format!("ENVIRONMENT: {}", state.environment)));

    // Version status
    if state.update_available {
        // Look for the most recent version checker event to determine color
        let mut version_color = Color::LightYellow; // Default fallback
        for event in state.events.iter().rev() {
            if matches!(event.worker, Worker::VersionChecker) {
                version_color = match (event.event_type, event.log_level) {
                    (EventType::Error, crate::error_classifier::LogLevel::Error) => Color::Red,
                    (EventType::Error, crate::error_classifier::LogLevel::Warn) => {
                        Color::LightYellow
                    }
                    (EventType::Success, _) => Color::Cyan,
                    _ => Color::LightYellow,
                };
                break;
            }
        }

        if let Some(latest) = &state.latest_version {
            let version_text = format!("VERSION: {} → {}", version, latest);
            status_lines.push(Line::from(vec![Span::styled(
                version_text,
                Style::default().fg(version_color),
            )]));
        } else {
            status_lines.push(Line::from(vec![Span::styled(
                "VERSION: Update Available",
                Style::default().fg(version_color),
            )]));
        }
    } else {
        status_lines.push(Line::from(format!("VERSION: {}", version)));
    }

    // Uptime in Days, Hours, Minutes, Seconds
    let uptime = state.start_time.elapsed();
    let uptime_string = format!(
        "UPTIME: {}d {}h {}m {}s",
        uptime.as_secs() / 86400,
        (uptime.as_secs() % 86400) / 3600,
        (uptime.as_secs() % 3600) / 60,
        uptime.as_secs() % 60
    );
    status_lines.push(Line::from(uptime_string));

    // NEX Points
    if let Some(nex_points) = state.nex_points {
        status_lines.push(Line::from(format!("NEX POINTS: {}", nex_points)));
    }

    // Current Task
    if let Some(task) = &state.current_task {
        status_lines.push(Line::from(format!("CURRENT TASK: {}", task)));
    }

    // Total Cores
    status_lines.push(Line::from(format!("TOTAL CORES: {}", state.total_cores)));

    // Total RAM in GB
    status_lines.push(Line::from(format!(
        "TOTAL RAM: {:.3} GB",
        state.total_ram_gb
    )));

    let status_paragraph = Paragraph::new(status_lines)
        .block(status_block)
        .style(Style::default().fg(Color::Cyan))
        .wrap(Wrap { trim: true });
    f.render_widget(status_paragraph, body_chunks[0]);

    // Logs Section
    let log_lines: Vec<Line> = state
        .events
        .iter()
        .filter(|event| event.should_display())
        .rev() // newest first
        .map(|event| {
            let main_icon = match (event.event_type, event.log_level) {
                (EventType::Success, _) => "✅",
                (EventType::Error, crate::error_classifier::LogLevel::Error) => "❌",
                (EventType::Error, crate::error_classifier::LogLevel::Warn) => "⚠️",
                (EventType::Error, _) => "❌",
                (EventType::Refresh, _) => "🔄",
                (EventType::Shutdown, _) => "🔴",
            };

            let worker_type = match event.worker {
                Worker::TaskFetcher => "Fetcher".to_string(),
                Worker::Prover(worker_id) => format!("P{}", worker_id),
                Worker::ProofSubmitter => "Submitter".to_string(),
                Worker::VersionChecker => "Version".to_string(),
            };

            let worker_color = DashboardState::get_worker_color(&event.worker);
            let compact_time = DashboardState::format_compact_timestamp(&event.timestamp);

            // Clean HTTP error messages
            let cleaned_msg = DashboardState::clean_http_error_message(&event.msg);

            // For version checker events, use appropriate color based on log level
            let message_color = if matches!(event.worker, Worker::VersionChecker) {
                match (event.event_type, event.log_level) {
                    (EventType::Error, crate::error_classifier::LogLevel::Error) => Color::Red,
                    (EventType::Error, crate::error_classifier::LogLevel::Warn) => {
                        Color::LightYellow
                    }
                    (EventType::Success, _) => Color::Cyan,
                    _ => worker_color,
                }
            } else {
                worker_color
            };

            // Create a structured line with colored spans
            Line::from(vec![
                // Main status icon
                Span::raw(format!("{} ", main_icon)),
                // Compact timestamp in muted color
                Span::styled(
                    format!("{} ", compact_time),
                    Style::default().fg(Color::DarkGray),
                ),
                // Worker type in bold with worker color
                Span::styled(
                    format!("[{}] ", worker_type),
                    Style::default()
                        .fg(worker_color)
                        .add_modifier(Modifier::BOLD),
                ),
                // Cleaned message with appropriate color
                Span::styled(cleaned_msg, Style::default().fg(message_color)),
            ])
        })
        .collect();

    let log_paragraph = if log_lines.is_empty() {
        Paragraph::new(vec![Line::from("Starting...")])
    } else {
        Paragraph::new(log_lines)
    };

    let log_widget = log_paragraph
        .block(
            Block::default().title("LOGS").borders(Borders::NONE).style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(log_widget, body_chunks[1]);

    // Footer with version info
    let footer_text = if state.update_available {
        "[Q] Quit | 🚀 New version available! Check release notes at github.com/nexus-xyz/nexus-cli"
    } else {
        "[Q] Quit"
    };

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}

#[cfg(test)]
mod tests {}
