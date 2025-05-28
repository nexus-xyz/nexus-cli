//! Dashboard screen rendering.

use crate::utils;
use crate::utils::system;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;
use std::time::Instant;

/// State for the dashboard screen, containing node information and menu items.
pub struct DashboardState {
    /// Unique identifier for the node.
    pub node_id: Option<u64>,

    /// Total NEX points available to the node, if any.
    pub nex_points: Option<u64>,

    /// The start time of the application, used for computing uptime.
    pub start_time: Instant,

    /// The current task being executed by the node, if any.
    pub current_task: Option<String>,

    /// Logs or main content area text.
    pub logs: String,
}

impl DashboardState {
    /// Creates a new instance of the dashboard state.
    ///
    /// # Arguments
    /// * `start_time` - The start time of the application, used for computing uptime.
    /// * `node_id` - Optional node ID for authenticated sessions.
    pub fn new(node_id: Option<u64>, start_time: Instant) -> Self {
        Self {
            node_id,
            nex_points: None,
            start_time,
            current_task: None,
            logs: "A scrolling history of event logs...".to_string(),
        }
    }

    /// Updates the dashboard state.
    pub fn update(&mut self) {
        todo!("Update the dashboard state with new data");
    }

    /// Render the dashboard state to the terminal frame.
    pub fn render(&self, f: &mut Frame) {
        render_dashboard(f, self);
    }
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
        items.push(ListItem::new(format!(
            "TOTAL CORES: {}",
            system::num_cores()
        )));

        // Total RAM in GB
        items.push(ListItem::new(format!(
            "TOTAL RAM: {:.2} GB",
            system::total_memory_gb()
        )));

        // CPU Load (Placeholder)
        items.push(ListItem::new("CPU LOAD: 0%".to_string())); // Placeholder, replace with actual data

        // RAM Used (Placeholder)
        items.push(ListItem::new("RAM USED: 0.00 GB".to_string())); // Placeholder, replace with actual data

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

    // Body: Main Content Area
    let content_text = state.logs.clone();
    let content_block = Block::default().borders(Borders::NONE).title("LOGS").style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    let content = Paragraph::new(content_text)
        .style(Style::default().fg(Color::Cyan))
        .block(content_block);
    f.render_widget(content, body_chunks[1]);

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
