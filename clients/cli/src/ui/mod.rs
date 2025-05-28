mod dashboard;
mod login;
mod splash;

use crate::ui::dashboard::{render_dashboard, DashboardState};
use crate::ui::login::render_login;
use crate::ui::splash::render_splash;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{backend::Backend, Frame, Terminal};
use std::time::{Duration, Instant};

/// The different screens in the application.
pub enum Screen {
    /// Splash screen shown at the start of the application.
    Splash,
    /// Login screen where users can authenticate.
    Login,
    /// Dashboard screen displaying node information and status.
    Dashboard(DashboardState),
}

/// Application state
pub struct App {
    /// The start time of the application, used for computing uptime.
    pub start_time: Instant,

    /// Optional node ID for authenticated sessions
    pub node_id: Option<u64>,

    /// The current screen being displayed in the application.
    pub current_screen: Screen,
}

impl App {
    /// Creates a new instance of the application.
    pub fn new(node_id: Option<u64>) -> Self {
        Self {
            node_id,
            current_screen: Screen::Splash,
            start_time: Instant::now(),
        }
    }

    /// Handles a complete login process, transitioning to the dashboard screen.
    pub fn login(&mut self) {
        let node_id = Some(123); // Placeholder for node ID, replace with actual logic to get node ID
        let state = DashboardState::new(node_id, self.start_time);
        self.current_screen = Screen::Dashboard(state);
    }
}

/// Runs the application UI in a loop, handling events and rendering the appropriate screen.
pub fn run<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> std::io::Result<()> {
    let splash_start = Instant::now();
    let splash_duration = Duration::from_secs(2);

    loop {
        terminal.draw(|f| render(f, &mut app))?;

        // Handle splash-to-login transition
        if let Screen::Splash = app.current_screen {
            if splash_start.elapsed() >= splash_duration {
                app.current_screen =
                    Screen::Dashboard(DashboardState::new(app.node_id, app.start_time));
                continue;
            }
        }

        // Don't block on event while in splash so that the splash screen can time out.
        if event::poll(Duration::from_millis(10))? {
            // Poll for key events
            if let Event::Key(key) = event::read()? {
                // Skip events that are not KeyEventKind::Press
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }

                // Handle exit events
                match key.code {
                    // Handle Escape and 'q' keys to exit the application
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }

                match &mut app.current_screen {
                    // Handle events for the Splash screen
                    &mut Screen::Splash => {
                        // Any key press will transition from the splash screen to the login screen
                        if key.code != KeyCode::Esc && key.code != KeyCode::Char('q') {
                            app.current_screen = Screen::Login;
                        }
                    }

                    // Handle events for the Login screen
                    Screen::Login => match key.code {
                        KeyCode::Enter => app.login(),
                        _ => {}
                    },

                    // Handle events for the Dashboard screen
                    Screen::Dashboard(_dashboard_state) => match key.code {
                        // KeyCode::Up => {
                        //     if main_state.selected_menu_index > 0 {
                        //         main_state.selected_menu_index -= 1;
                        //     }
                        // }
                        // KeyCode::Down => {
                        //     if main_state.selected_menu_index + 1 < main_state.status_items.len() {
                        //         main_state.selected_menu_index += 1;
                        //     }
                        // }
                        _ => {}
                    },
                }
            }
        }
    }
}

/// Renders the current screen based on the application state.
fn render(f: &mut Frame, app: &App) {
    match &app.current_screen {
        Screen::Splash => render_splash(f),
        Screen::Login => render_login(f),
        Screen::Dashboard(state) => render_dashboard(f, state),
    }
}
