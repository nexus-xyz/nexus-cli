mod dashboard;
mod login;
mod splash;

use crate::environment::Environment;
use crate::orchestrator::{Orchestrator, OrchestratorClient};
use crate::prover_runtime::{start_anonymous_workers, start_authenticated_workers, ProverEvent};
use crate::ui::dashboard::{render_dashboard, DashboardState};
use crate::ui::login::render_login;
use crate::ui::splash::render_splash;
use crossterm::event::{self, Event, KeyCode};
use ed25519_dalek::SigningKey;
use ratatui::{backend::Backend, Frame, Terminal};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

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
#[derive(Debug, Clone)]
pub struct App {
    /// The start time of the application, used for computing uptime.
    pub start_time: Instant,

    /// Optional node ID for authenticated sessions
    pub node_id: Option<u64>,

    /// The environment in which the application is running.
    pub environment: Environment,

    /// The client used to interact with the Nexus Orchestrator.
    pub orchestrator_client: OrchestratorClient,

    /// The current screen being displayed in the application.
    pub current_screen: Screen,

    /// Events received from worker threads.
    pub events: VecDeque<ProverEvent>,

    /// Proof-signing key.
    signing_key: SigningKey,
}

impl App {
    /// Creates a new instance of the application.
    pub fn new(
        node_id: Option<u64>,
        orchestrator_client: OrchestratorClient,
        signing_key: SigningKey,
    ) -> Self {
        Self {
            start_time: Instant::now(),
            node_id,
            environment: *orchestrator_client.environment(),
            orchestrator_client,
            current_screen: Screen::Splash,
            events: Default::default(),
            signing_key,
        }
    }

    /// Handles a complete login process, transitioning to the dashboard screen.
    #[allow(unused)]
    pub fn login(&mut self) {
        let node_id = Some(123); // Placeholder for node ID, replace with actual logic to get node ID
        let state = DashboardState::new(node_id, self.environment, self.start_time, &self.events);
        self.current_screen = Screen::Dashboard(state);
    }
}

/// Runs the application UI in a loop, handling events and rendering the appropriate screen.
pub async fn run<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> std::io::Result<()> {
    let splash_start = Instant::now();
    let splash_duration = Duration::from_secs(2);

    let num_workers = 2; // TODO: Keep this low for now to avoid hitting rate limits.

    // Receives events from prover worker threads.
    let mut prover_event_receiver = match app.node_id {
        Some(node_id) => start_authenticated_workers(
            node_id,
            app.signing_key.clone(),
            app.orchestrator_client.clone(),
            num_workers,
        ),
        None => start_anonymous_workers(num_workers),
    };

    // UI event loop
    loop {
        // Drain prover events from the async channel into app.events
        while let Ok(event) = prover_event_receiver.try_recv() {
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
                let state =
                    DashboardState::new(app.node_id, app.environment, app.start_time, &app.events);
                app.current_screen = Screen::Dashboard(state);
            }
        }
        terminal.draw(|f| render(f, &app.current_screen))?;

        // Handle splash-to-login transition
        if let Screen::Splash = app.current_screen {
            if splash_start.elapsed() >= splash_duration {
                app.current_screen = Screen::Dashboard(DashboardState::new(
                    app.node_id,
                    app.environment,
                    app.start_time,
                    &app.events,
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
                    // TODO: Close worker threads
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
                                &app.events,
                            ));
                        }
                    }
                    Screen::Login => {
                        todo!()
                        // if key.code == KeyCode::Enter {
                        //     app.login();
                        // }
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
