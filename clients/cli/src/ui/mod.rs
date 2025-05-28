mod dashboard;
mod login;
mod splash;

use crate::environment::Environment;
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

    /// The environment in which the application is running.
    pub environment: Environment,

    /// The current screen being displayed in the application.
    pub current_screen: Screen,
}

impl App {
    /// Creates a new instance of the application.
    pub fn new(node_id: Option<u64>, env: Environment) -> Self {
        Self {
            start_time: Instant::now(),
            node_id,
            environment: env,
            current_screen: Screen::Splash,
        }
    }

    /// Handles a complete login process, transitioning to the dashboard screen.
    pub fn login(&mut self) {
        let node_id = Some(123); // Placeholder for node ID, replace with actual logic to get node ID
        let state = DashboardState::new(node_id, self.environment, self.start_time);
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
                app.current_screen = Screen::Dashboard(DashboardState::new(
                    app.node_id,
                    app.environment,
                    app.start_time,
                ));
                continue;
            }
        }

        // Poll for key events
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Skip events that are not KeyEventKind::Press
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }

                // Handle exit events
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                    return Ok(());
                }

                match &mut app.current_screen {
                    Screen::Splash => {
                        // Any key press will skip the splash screen
                        if key.code != KeyCode::Esc && key.code != KeyCode::Char('q') {
                            app.current_screen = Screen::Dashboard(DashboardState::new(
                                app.node_id,
                                app.environment,
                                app.start_time,
                            ));
                        }
                    }
                    Screen::Login => match key.code {
                        KeyCode::Enter => app.login(),
                        _ => {}
                    },
                    Screen::Dashboard(_dashboard_state) => match key.code {
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
