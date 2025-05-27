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
    Login,
    Main(MainScreenState),
}

/// Application state for the main screen.
pub struct MainScreenState {
    pub node_id: u64,
    pub menu_items: Vec<String>,
    pub selected_menu_index: usize,
    pub content: String,
}

impl MainScreenState {
    pub fn new() -> Self {
        let node_id = 1234; // This would typically be fetched from a configuration or environment variable.
        Self {
            node_id,
            menu_items: vec![format!("NODE ID: {}", node_id).to_string(), "CORES:".to_string(), "MEMORY: (GB)".to_string(), "GFLOP/s: ".to_string()],
            selected_menu_index: 0,
            content: "Welcome to the main content area.".to_string(),
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
        let state = MainScreenState::new();
        self.current_screen = Screen::Main(state);
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
                // Handle events for the login screen
                Screen::Login => match key.code {
                    KeyCode::Enter => app.login(),
                    KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                // Handle events for the main screen
                Screen::Main(main_state) => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => {
                        if main_state.selected_menu_index > 0 {
                            main_state.selected_menu_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if main_state.selected_menu_index + 1 < main_state.menu_items.len() {
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
        Screen::Login => render_login(f),
        Screen::Main(state) => render_main(f, state),
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
fn render_main(f: &mut Frame, state: &MainScreenState) {
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

    // Body layout
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)].as_ref())
        .split(chunks[1]);

    // // Body: Left Menu
    // let menu = Paragraph::new("Menu\n- Option 1\n- Option 2\n- Option 3")
    //     .style(Style::default().fg(Color::Cyan))
    //     .block(Block::default().borders(Borders::RIGHT));
    // f.render_widget(menu, body_chunks[0]);

    // --- Left Menu using List ---
    // let menu_items = vec!["Option 1", "Option 2", "Option 3"];
    let items: Vec<ListItem> = state.menu_items
        .iter()
        .map(|i| ListItem::new(i.clone()))
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_menu_index));

    let menu = List::new(items)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::RIGHT).title("NODE STATS").style(Style::default().add_modifier(Modifier::BOLD)))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(menu, body_chunks[0], &mut list_state);

    // Body: Main Content Area
    let content_text = state.content.clone();
    let content = Paragraph::new(content_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(content, body_chunks[1]);

    // Footer
    let footer = Paragraph::new("Keyboard: [q] Quit | [←][→] Navigate")
        .alignment(Alignment::Center) // ← Horizontally center the text
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}
