//! Dashboard screen rendering.

use crate::environment::Environment;
use crate::events::{Event as WorkerEvent, EventType, Worker};
use crate::system;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
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
}

impl DashboardState {
    /// Creates a new instance of the dashboard state.
    ///
    /// # Arguments
    /// * `node_id` - This node's unique identifier, if available.
    /// * `start_time` - The start time of the application, used for computing uptime.
    /// * `environment` - The environment in which the application is running.
    pub fn new(
        node_id: Option<u64>,
        environment: Environment,
        start_time: Instant,
        events: &VecDeque<WorkerEvent>,
    ) -> Self {
        // Check for version update messages in recent events
        let (update_available, latest_version) = Self::check_for_version_updates(events);

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
        }
    }

    /// Check recent events for version update information
    fn check_for_version_updates(events: &VecDeque<WorkerEvent>) -> (bool, Option<String>) {
        // Look for the most recent version checker success event
        for event in events.iter().rev() {
            if matches!(event.worker, Worker::VersionChecker)
                && event.event_type == EventType::Success
            {
                // Parse the version from the message
                if let Some(version) = Self::extract_version_from_message(&event.msg) {
                    return (true, Some(version));
                }
            }
        }
        (false, None)
    }

    /// Extract version number from version checker message
    fn extract_version_from_message(message: &str) -> Option<String> {
        // Look for pattern like "New version v0.9.1 available!"
        if let Some(start) = message.find("version ") {
            let after_version = &message[start + 8..];
            if let Some(end) = after_version.find(" available") {
                return Some(after_version[..end].to_string());
            }
        }
        None
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

    /// Truncate long messages to prevent layout overflow
    fn truncate_message(msg: &str, max_length: usize) -> String {
        if msg.len() <= max_length {
            msg.to_string()
        } else {
            // Try to truncate at word boundary if possible
            let truncate_target = max_length.saturating_sub(3);
            if let Some(last_space) = msg[..truncate_target].rfind(' ') {
                format!("{}...", &msg[..last_space])
            } else {
                format!("{}...", &msg[..truncate_target])
            }
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

// const SUCCESS_ICON: &str = "✅";
// const ERROR_ICON: &str = "⚠️";
// const REFRESH_ICON: &str = "🔄";
// const SHUTDOWN_ICON: &str = "🔴";

/// Render the dashboard screen.
pub fn render_dashboard(f: &mut Frame, state: &DashboardState) {
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
        Color::LightYellow // Highlight when update is available
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
    // Make status column slightly wider to accommodate more text
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(28), Constraint::Percentage(72)].as_ref())
        .split(chunks[1]);

    // --- Status using List ---
    let mut status_list_state = ListState::default();
    let status: List = {
        let status_block = Block::default()
            .borders(Borders::RIGHT)
            .title("STATUS")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        let mut items: Vec<ListItem> = Vec::new();

        // Display the node ID, if any, or "Not connected" if not available
        let node_id_text = if let Some(id) = state.node_id {
            format!("NODE ID: {}", id)
        } else {
            "NODE ID: Not connected".to_string()
        };
        items.push(ListItem::new(node_id_text));

        // Environment
        items.push(ListItem::new(format!("ENVIRONMENT: {}", state.environment)));

        // Version status
        if state.update_available {
            if let Some(latest) = &state.latest_version {
                let version_text = format!("VERSION: {} → {} 🚀", version, latest);
                items.push(ListItem::new(Line::from(vec![Span::styled(
                    version_text,
                    Style::default().fg(Color::LightYellow),
                )])));
            } else {
                items.push(ListItem::new(Line::from(vec![Span::styled(
                    "VERSION: Update Available 🚀",
                    Style::default().fg(Color::LightYellow),
                )])));
            }
        } else {
            items.push(ListItem::new(format!("VERSION: {}", version)));
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
        items.push(ListItem::new(uptime_string));

        // NEX Points
        if let Some(nex_points) = state.nex_points {
            items.push(ListItem::new(format!("NEX POINTS: {}", nex_points)));
        }

        // Current Task
        if let Some(task) = &state.current_task {
            items.push(ListItem::new(format!("CURRENT TASK: {}", task)));
        }

        // Total Cores
        items.push(ListItem::new(format!("TOTAL CORES: {}", state.total_cores)));

        // Total RAM in GB
        items.push(ListItem::new(format!(
            "TOTAL RAM: {:.3} GB",
            state.total_ram_gb
        )));

        List::new(items)
            .style(Style::default().fg(Color::Cyan))
            .block(status_block)
            .highlight_style(Style::default().fg(Color::Cyan))
            .highlight_symbol("> ")
    };
    f.render_stateful_widget(status, body_chunks[0], &mut status_list_state);

    // Create styled log items with enhanced UX and visual improvements
    let log_items: Vec<ListItem> = state
        .events
        .iter()
        .filter(|event| event.should_display())
        .rev() // newest first
        .map(|event| {
            let main_icon = match event.event_type {
                EventType::Success => "✅",
                EventType::Error => "❌",
                EventType::Refresh => "🔄",
                EventType::Shutdown => "🔴",
            };

            let worker_type = match event.worker {
                Worker::TaskFetcher => "Fetcher".to_string(),
                Worker::Prover(worker_id) => format!("P{}", worker_id),
                Worker::ProofSubmitter => "Submitter".to_string(),
                Worker::VersionChecker => "Version".to_string(),
            };

            let worker_color = DashboardState::get_worker_color(&event.worker);
            let compact_time = DashboardState::format_compact_timestamp(&event.timestamp);

            // Clean HTTP error messages first, then truncate if needed
            let cleaned_msg = DashboardState::clean_http_error_message(&event.msg);
            let final_msg = DashboardState::truncate_message(&cleaned_msg, 120);

            // Create a more structured layout with better visual hierarchy
            let line = Line::from(vec![
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
                // Cleaned and truncated message in worker color
                Span::styled(final_msg, Style::default().fg(worker_color)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let final_log_items = if log_items.is_empty() {
        vec![ListItem::new("Starting...")]
    } else {
        log_items
    };

    let log_widget = List::new(final_log_items)
        .block(Block::default().title("LOGS").borders(Borders::NONE))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

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
