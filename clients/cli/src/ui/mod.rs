mod dashboard;
mod login;
pub mod splash;

use crate::environment::Environment;
use crate::events::Event as WorkerEvent;
use crate::ui::dashboard::{DashboardState, render_dashboard};
use crate::ui::login::render_login;
use crate::ui::splash::render_splash;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{Frame, Terminal, backend::Backend};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};

/// The different screens in the application.
#[derive(Debug, Clone)]
pub enum Screen {
    /// Splash screen shown at the start of the application.
    Splash,
    /// Login screen where users can authenticate.
    #[allow(unused)]
    Login,
    /// Dashboard screen displaying node information and status.
    Dashboard(DashboardState),
}

/// The maximum number of events to keep in the event buffer.
const MAX_EVENTS: usize = 100;

/// Application state
#[derive(Debug)]
pub struct App {
    /// The start time of the application, used for computing uptime.
    start_time: Instant,

    /// Optional node ID for authenticated sessions
    node_id: Option<u64>,

    /// The environment in which the application is running.
    environment: Environment,

    /// The current screen being displayed in the application.
    current_screen: Screen,

    /// Events received from worker threads.
    events: VecDeque<WorkerEvent>,

    /// Receives events from worker threads.
    event_receiver: mpsc::Receiver<WorkerEvent>,

    /// Broadcasts shutdown signal to worker threads.
    shutdown_sender: broadcast::Sender<()>,

    /// Whether to disable background colors
    no_background_color: bool,
}

impl App {
    /// Creates a new instance of the application.
    pub fn new(
        node_id: Option<u64>,
        environment: Environment,
        event_receiver: mpsc::Receiver<WorkerEvent>,
        shutdown_sender: broadcast::Sender<()>,
        no_background_color: bool,
    ) -> Self {
        Self {
            start_time: Instant::now(),
            node_id,
            environment,
            current_screen: Screen::Splash,
            events: Default::default(),
            event_receiver,
            shutdown_sender,
            no_background_color,
        }
    }

    /// Handles a complete login process, transitioning to the dashboard screen.
    #[allow(unused)]
    pub fn login(&mut self) {
        let node_id = Some(123); // Placeholder for node ID, replace with actual logic to get node ID
        let state = DashboardState::new(
            node_id,
            self.environment.clone(),
            self.start_time,
            &self.events,
            self.no_background_color,
        );
        self.current_screen = Screen::Dashboard(state);
    }
}

/// Runs the application UI in a loop, handling events and rendering the appropriate screen.
pub async fn run<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> std::io::Result<()> {
    let splash_start = Instant::now();
    let splash_duration = Duration::from_secs(2);

    // UI event loop
    loop {
        // Drain prover events from the async channel into app.events
        while let Ok(event) = app.event_receiver.try_recv() {
            if app.events.len() >= MAX_EVENTS {
                app.events.pop_front();
            }
            app.events.push_back(event);
        }

        // Update the state based on the current screen
        match app.current_screen {
            Screen::Splash => {}
            Screen::Login => {}
            Screen::Dashboard(_) => {
                let state = DashboardState::new(
                    app.node_id,
                    app.environment.clone(),
                    app.start_time,
                    &app.events,
                    app.no_background_color,
                );
                app.current_screen = Screen::Dashboard(state);
            }
        }
        terminal.draw(|f| render(f, &app.current_screen))?;

        // Handle splash-to-login transition
        if let Screen::Splash = app.current_screen {
            if splash_start.elapsed() >= splash_duration {
                app.current_screen = Screen::Dashboard(DashboardState::new(
                    app.node_id,
                    app.environment.clone(),
                    app.start_time,
                    &app.events,
                    app.no_background_color,
                ));
                continue;
            }
        }

        // Poll for key events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Skip events that are not KeyEventKind::Press
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }

                // Handle exit events
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                    // Send shutdown signal to workers
                    let _ = app.shutdown_sender.send(());
                    return Ok(());
                }

                match &mut app.current_screen {
                    Screen::Splash => {
                        // Any key press will skip the splash screen
                        if key.code != KeyCode::Esc && key.code != KeyCode::Char('q') {
                            app.current_screen = Screen::Dashboard(DashboardState::new(
                                app.node_id,
                                app.environment.clone(),
                                app.start_time,
                                &app.events,
                                app.no_background_color,
                            ));
                        }
                    }
                    Screen::Login => {
                        if key.code == KeyCode::Enter {
                            app.login();
                        }
                    }
                    Screen::Dashboard(_dashboard_state) => {}
                }
            }
        }
    }
}

/// Renders the current screen based on the application state.
fn render(f: &mut Frame, screen: &Screen) {
    match screen {
        Screen::Splash => render_splash(f),
        Screen::Login => render_login(f),
        Screen::Dashboard(state) => render_dashboard(f, state),
    }
}
