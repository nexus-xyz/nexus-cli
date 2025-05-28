//! Dashboard screen rendering.

use crate::environment::Environment;
use crate::ui::WorkerEvent;
use crate::utils::system;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;
use std::collections::VecDeque;
use std::time::Instant;

/// State for the dashboard screen, containing node information and menu items.
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

    // /// Logs or messages to display in the dashboard.
    // pub logs: Vec<String>,
    /// Total number of (virtual) CPU cores available on the machine.
    pub total_cores: usize,

    /// Total RAM available on the machine, in GB.
    pub total_ram_gb: f64,

    pub events: VecDeque<WorkerEvent>,
}

impl DashboardState {
    /// Creates a new instance of the dashboard state.
    ///
    /// # Arguments
    /// * `start_time` - The start time of the application, used for computing uptime.
    /// * `environment` - The environment in which the application is running.
    /// * `node_id` - Optional node ID for authenticated sessions.
    pub fn new(
        node_id: Option<u64>,
        environment: Environment,
        start_time: Instant,
        events: &VecDeque<WorkerEvent>,
    ) -> Self {
        // let logs = vec![
        //     "[12:48:11] ✅ Proof accepted (23ms)".to_string(),
        //     "[12:48:09] ⚠️  Task stalled".to_string(),
        //     "[12:47:50] ✅ Proof accepted (22ms)".to_string(),
        // ];

        Self {
            node_id,
            environment,
            nex_points: None,
            start_time,
            current_task: None,
            // logs,
            total_cores: system::num_cores(),
            total_ram_gb: system::total_memory_gb(),
            events: events.clone(),
        }
    }

    // /// Updates the dashboard state.
    // pub fn update(&mut self) {
    //     self.logs.push(format!("Heartbeat at {:?}", Instant::now()));
    // }

    // /// Render the dashboard state to the terminal frame.
    // pub fn render(&self, f: &mut Frame) {
    //     render_dashboard(f, self);
    // }
}

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
        .split(f.size());

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
    // status_list_state.select(Some(state.selected_menu_index));
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

        // Node ID
        items.push(ListItem::new(format!(
            "NODE ID: {}",
            state.node_id.unwrap_or(0)
        )));

        // Environment
        items.push(ListItem::new(format!(
            "ENVIRONMENT: {}",
            state.environment.to_string()
        )));

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
        } else {
            items.push(ListItem::new("NEX POINTS: Not available".to_string()));
        }

        // Current Task
        if let Some(task) = &state.current_task {
            items.push(ListItem::new(format!("CURRENT TASK: {}", task)));
        } else {
            items.push(ListItem::new("CURRENT TASK: None".to_string()));
        }

        // Total Cores
        items.push(ListItem::new(format!("TOTAL CORES: {}", state.total_cores)));

        // Total RAM in GB
        items.push(ListItem::new(format!(
            "TOTAL RAM: {:.3} GB",
            state.total_ram_gb
        )));

        // CPU Load (Placeholder)
        items.push(ListItem::new("CPU LOAD: 0.000%".to_string())); // Placeholder, replace with actual data

        // // RAM Used
        // items.push(ListItem::new(format!(
        //     "RAM USED: {:.3} GB",
        //     system::process_memory_gb()
        // )));

        List::new(items)
            .style(Style::default().fg(Color::Cyan))
            .block(status_block)
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ")
    };
    f.render_stateful_widget(status, body_chunks[0], &mut status_list_state);

    let logs: Vec<String> = state
        .events
        .iter()
        .map(|event| match event {
            WorkerEvent::Message { worker_id, data } => {
                format!("[{}] {}", worker_id, data)
            }
            WorkerEvent::Done { worker_id } => {
                format!("[{}] Task completed", worker_id)
            }
        })
        .collect();

    // Logs using List
    let log_items: Vec<ListItem> = logs
        .iter()
        .rev() // newest first
        .map(|line| ListItem::new(line.clone()))
        .collect();

    let log_widget = List::new(log_items)
        .block(Block::default().title("LOGS").borders(Borders::NONE))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(log_widget, body_chunks[1]);

    // Footer
    let footer = Paragraph::new("[Q] Quit  [S] Settings  [←][→] Navigate")
        .alignment(Alignment::Center) // ← Horizontally center the text
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}
