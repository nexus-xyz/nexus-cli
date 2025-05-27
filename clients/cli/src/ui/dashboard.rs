//! Dashboard screen rendering.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

/// State for the dashboard screen, containing node information and menu items.
pub struct DashboardState {
    /// Unique identifier for the node.
    pub node_id: Option<u64>,

    /// List of status items to display in the dashboard.
    pub status_items: Vec<String>,

    /// Index of the currently selected menu item.
    pub selected_menu_index: usize,

    /// Logs or main content area text.
    pub logs: String,
}

impl DashboardState {
    pub fn new(node_id: Option<u64>) -> Self {
        Self {
            node_id,
            status_items: vec![
                format!("NODE ID: {}", node_id.unwrap_or(0)).to_string(), // TODO
                "NEX POINTS:".to_string(),
                "UPTIME:".to_string(),
                "TOTAL CORES:".to_string(),
                "TOTAL RAM: (GB)".to_string(),
                "DEDICATED THREADS:".to_string(),
                "CURRENT TASK: foo.elf".to_string(),
                "CPU LOAD:".to_string(),
                "RAM USED: (GB)".to_string(),
            ],
            selected_menu_index: 0,
            logs: "A scrolling history of event logs...".to_string(),
        }
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
    status_list_state.select(Some(state.selected_menu_index));
    let status: List = {
        let status_block = Block::default()
            .borders(Borders::RIGHT)
            .title("STATUS")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        let items: Vec<ListItem> = state
            .status_items
            .iter()
            .map(|i| ListItem::new(i.clone()))
            .collect();

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
