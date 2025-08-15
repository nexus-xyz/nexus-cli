//! Main application state and UI loop
//!
//! Contains the App struct and main UI event handling logic

use crate::environment::Environment;
use crate::events::Event as WorkerEvent;
use crate::ui::dashboard::{DashboardState, render_dashboard};
use crate::ui::login::render_login;
use crate::ui::splash::render_splash;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{Frame, Terminal, backend::Backend};
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};

/// UI configuration data grouped by concern
#[derive(Debug, Clone)]
pub struct UIConfig {
    pub with_background_color: bool,
    pub num_threads: usize,
    pub update_available: bool,
    pub latest_version: Option<String>,
}

impl UIConfig {
    pub fn new(
        with_background_color: bool,
        num_threads: usize,
        update_available: bool,
        latest_version: Option<String>,
    ) -> Self {
        Self {
            with_background_color,
            num_threads,
            update_available,
            latest_version,
        }
    }
}

/// The different screens in the application.
#[derive(Debug)]
pub enum Screen {
    /// Splash screen shown at the start of the application.
    Splash,
    /// Login screen where users can authenticate.
    #[allow(unused)]
    Login,
    /// Dashboard screen displaying node information and status.
    Dashboard(Box<DashboardState>),
}

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

    /// Receives events from worker threads.
    event_receiver: mpsc::Receiver<WorkerEvent>,

    /// Broadcasts shutdown signal to worker threads.
    shutdown_sender: broadcast::Sender<()>,

    /// Receives max tasks completion signal.
    max_tasks_shutdown_receiver: broadcast::Receiver<()>,

    /// Whether to disable background colors
    with_background_color: bool,

    /// Number of worker threads being used for proving.
    num_threads: usize,

    /// Whether a version update is available.
    version_update_available: bool,

    /// Latest version available, if any.
    latest_version: Option<String>,
}

impl App {
    /// Creates a new instance of the application.
    pub fn new(
        node_id: Option<u64>,
        environment: Environment,
        event_receiver: mpsc::Receiver<WorkerEvent>,
        shutdown_sender: broadcast::Sender<()>,
        max_tasks_shutdown_receiver: broadcast::Receiver<()>,
        ui_config: UIConfig,
    ) -> Self {
        Self {
            start_time: Instant::now(),
            node_id,
            environment,
            current_screen: Screen::Splash,
            event_receiver,
            shutdown_sender,
            max_tasks_shutdown_receiver,
            with_background_color: ui_config.with_background_color,
            num_threads: ui_config.num_threads,
            version_update_available: ui_config.update_available,
            latest_version: ui_config.latest_version,
        }
    }

    /// Handles a complete login process, transitioning to the dashboard screen.
    #[allow(unused)]
    pub fn login(&mut self) {
        let node_id = Some(123); // Placeholder for node ID, replace with actual logic to get node ID
        let ui_config = UIConfig::new(
            self.with_background_color,
            self.num_threads,
            self.version_update_available,
            self.latest_version.clone(),
        );
        let state = DashboardState::new(
            node_id,
            self.environment.clone(),
            self.start_time,
            ui_config,
        );
        self.current_screen = Screen::Dashboard(Box::new(state));
    }
}

/// Runs the application UI in a loop, handling events and rendering the appropriate screen.
pub async fn run<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> std::io::Result<()> {
    let splash_start = Instant::now();
    let splash_duration = Duration::from_secs(2);

    // UI event loop
    loop {
        // Check for max tasks completion signal (non-blocking)
        if app.max_tasks_shutdown_receiver.try_recv().is_ok() {
            // Send shutdown signal to workers and exit
            let _ = app.shutdown_sender.send(());
            return Ok(());
        }

        // Queue all incoming events for processing
        while let Ok(event) = app.event_receiver.try_recv() {
            // Add event to dashboard queue if it exists
            if let Screen::Dashboard(state) = &mut app.current_screen {
                state.add_event(event);
            }
        }

        // Update the state based on the current screen
        match &mut app.current_screen {
            Screen::Splash => {}
            Screen::Login => {}
            Screen::Dashboard(state) => {
                // Update the dashboard with new tick and metrics
                state.update();
            }
        }
        terminal.draw(|f| render(f, &app.current_screen))?;

        // Handle splash-to-login transition
        if let Screen::Splash = app.current_screen {
            if splash_start.elapsed() >= splash_duration {
                let ui_config = UIConfig::new(
                    app.with_background_color,
                    app.num_threads,
                    app.version_update_available,
                    app.latest_version.clone(),
                );
                app.current_screen = Screen::Dashboard(Box::new(DashboardState::new(
                    app.node_id,
                    app.environment.clone(),
                    app.start_time,
                    ui_config,
                )));
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
                            let ui_config = UIConfig::new(
                                app.with_background_color,
                                app.num_threads,
                                app.version_update_available,
                                app.latest_version.clone(),
                            );
                            app.current_screen = Screen::Dashboard(Box::new(DashboardState::new(
                                app.node_id,
                                app.environment.clone(),
                                app.start_time,
                                ui_config,
                            )));
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
