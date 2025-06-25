//! Dashboard screen rendering.

use crate::environment::Environment;
use crate::prover_runtime::{Event as WorkerEvent, EventType};
use crate::system;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Color, Modifier, Style};
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
        Self {
            node_id,
            environment,
            nex_points: None,
            start_time,
            current_task: None,
            total_cores: system::num_cores(),
            total_ram_gb: system::total_memory_gb(),
            events: events.clone(),
        }
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

    // Title section
    let version = env!("CARGO_PKG_VERSION");
    let title_text = format!("=== NEXUS PROVER v{} ===", version);
    let title_block = Block::default().borders(Borders::BOTTOM);
    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center) // ← Horizontally center the text
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(title_block);
    f.render_widget(title, chunks[0]);

    // Body layout: Split into two columns (status and logs)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)].as_ref())
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

    let logs: Vec<String> = state
        .events
        .iter()
        .map(|event| {
            let icon = match event.event_type {
                EventType::Success => "✅",
                EventType::Error => "⚠️",
                EventType::Refresh => "🔄",
                EventType::Shutdown => "🔴",
            };
            format!("{} [{}] {}", icon, event.timestamp, event.msg)
        })
        .collect();

    // Logs using List
    let mut log_items: Vec<ListItem> = logs
        .iter()
        .rev() // newest first
        .map(|line| ListItem::new(line.clone()))
        .collect();

    if log_items.is_empty() {
        log_items.push(ListItem::new("Starting...".to_string()));
    }

    let log_widget = List::new(log_items)
        .block(Block::default().title("LOGS").borders(Borders::NONE))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(log_widget, body_chunks[1]);

    // Footer
    let footer = Paragraph::new("[Q] Quit")
        .alignment(Alignment::Center) // ← Horizontally center the text
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}
