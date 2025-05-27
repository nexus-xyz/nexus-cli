use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use crossterm::event::{self, Event, KeyCode};
use ratatui::widgets::{List, ListItem, ListState};

/// Represents the different screens in the application.
pub enum Screen {
    Splash,
    Login,
    Dashboard(DashboardState),
}

/// State for the dashboard screen, containing node information and menu items.
pub struct DashboardState {
    /// Unique identifier for the node.
    pub node_id: u64,

    /// List of status items to display in the dashboard.
    pub status_items: Vec<String>,

    /// Index of the currently selected menu item.
    pub selected_menu_index: usize,

    /// Logs or main content area text.
    pub logs: String,
}

impl DashboardState {
    pub fn new(node_id: u64) -> Self {
        Self {
            node_id,
            status_items: vec![
                format!("NODE ID: {}", node_id).to_string(),
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

/// Application state
pub struct App {
    pub current_screen: Screen,
    // TODO: prover state
    // node_id: u64,
    // System info
    // Total NEX points
    // etc
}

impl App {
    /// Creates a new instance of the application.
    pub fn new() -> Self {
        Self {
            current_screen: Screen::Login,
        }
    }

    /// Handles the login action.
    pub fn login(&mut self) {
        let node_id = 123; // Placeholder for node ID, replace with actual logic to get node ID
        let state = DashboardState::new(node_id);
        self.current_screen = Screen::Dashboard(state);
    }
}

/// Runs the application UI in a loop, handling events and rendering the appropriate screen.
pub fn run<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> std::io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Poll for key events
        if let Event::Key(key) = event::read()? {
            if  key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match &mut app.current_screen {
                // Handle events for the splash screen
                &mut Screen::Splash => todo!(),
                // Handle events for the login screen
                Screen::Login => match key.code {
                    KeyCode::Enter => app.login(),
                    KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                // Handle events for the main screen
                Screen::Dashboard(main_state) => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => {
                        if main_state.selected_menu_index > 0 {
                            main_state.selected_menu_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if main_state.selected_menu_index + 1 < main_state.status_items.len() {
                            main_state.selected_menu_index += 1;
                        }
                    }
                    _ => {}
                },
            }
        }
    }
}

/// Renders the current screen based on the application state.
fn ui(f: &mut Frame, app: &App) {
    match &app.current_screen {
        Screen::Splash => todo!(), // Placeholder for splash screen rendering
        Screen::Login => render_login(f),
        Screen::Dashboard(state) => render_main(f, state),
    }
}

/// Renders the login screen with a simple message and instructions.
fn render_login(f: &mut Frame) {
    let size = f.size();

    let block = Block::default()
        .title("Login")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new("Press Enter to login\nPress Esc to exit")
        .block(block);

    f.render_widget(paragraph, size);
}

/// Renders the main screen with a title, menu, and content area.
fn render_main(f: &mut Frame, state: &DashboardState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title block
            Constraint::Min(0),     // Body area
            Constraint::Length(2),  // Footer block
        ].as_ref())
        .split(f.size());

    // Title section
    let version = env!("CARGO_PKG_VERSION");
    let title_text = format!("=== NEXUS PROVER v{} ===", version);
    let title_block = Block::default().borders(Borders::BOTTOM);
    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center) // ← Horizontally center the text
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
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
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

        let items: Vec<ListItem> = state.status_items
            .iter()
            .map(|i| ListItem::new(i.clone()))
            .collect();

        List::new(items)
            .style(Style::default().fg(Color::Cyan))
            .block(status_block)
            .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .highlight_symbol("> ")
    };
    f.render_stateful_widget(status, body_chunks[0], &mut status_list_state);

    // Body: Main Content Area
    let content_text = state.logs.clone();
    let content_block = Block::default()
        .borders(Borders::NONE)
        .title("LOGS")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    let content = Paragraph::new(content_text)
        .style(Style::default().fg(Color::Cyan))
        .block(content_block);
    f.render_widget(content, body_chunks[1]);

    // Footer
    let footer = Paragraph::new("[Q] Quit  [S] Settings  [←][→] Navigate")
        .alignment(Alignment::Center) // ← Horizontally center the text
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}
