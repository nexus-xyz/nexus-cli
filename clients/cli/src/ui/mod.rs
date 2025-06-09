mod dashboard;
mod login;
mod splash;

use crate::environment::Environment;
use crate::orchestrator::{Orchestrator, OrchestratorClient};
use crate::prover::{authenticated_proving, prove_anonymously};
use crate::ui::dashboard::{DashboardState, render_dashboard};
use crate::ui::login::render_login;
use crate::ui::splash::render_splash;
use chrono::Local;
use crossbeam::channel::unbounded;
use crossterm::event::{self, Event, KeyCode};
use ed25519_dalek::SigningKey;
use ratatui::{Frame, Terminal, backend::Backend};
use std::collections::VecDeque;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

/// The different screens in the application.
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
    pub fn login(&mut self) {
        let node_id = Some(123); // Placeholder for node ID, replace with actual logic to get node ID
        let state = DashboardState::new(node_id, self.environment, self.start_time, &self.events);
        self.current_screen = Screen::Dashboard(state);
    }
}

/// Runs the application UI in a loop, handling events and rendering the appropriate screen.
pub fn run<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> std::io::Result<()> {
    let splash_start = Instant::now();
    let splash_duration = Duration::from_secs(2);

    // Spawn worker threads for background tasks
    let num_workers = 1; // TODO: Keep this low for now to avoid hitting rate limits.
    let mut workers: Vec<JoinHandle<()>> = Vec::with_capacity(num_workers);
    let (sender, receiver) = unbounded::<ProverEvent>();
    for worker_id in 0..num_workers {
        let handle = match app.node_id {
            Some(node_id) => spawn_prover(
                worker_id,
                node_id,
                app.orchestrator_client.clone(),
                app.signing_key.clone(),
                sender.clone(),
            ),
            None => spawn_anonymous_prover(worker_id, sender.clone()),
        };

        workers.push(handle);
    }
    drop(sender); // Drop original sender to allow receiver to detect end-of-stream.
    let mut active_workers = num_workers;

    loop {
        terminal.draw(|f| render(f, &app))?;

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
                        if key.code == KeyCode::Enter {
                            app.login();
                        }
                    }
                    Screen::Dashboard(_dashboard_state) => {}
                }
            }
        }

        if active_workers > 0 {
            while let Ok(event) = receiver.try_recv() {
                // If Done, decrement active_workers
                if let ProverEvent::Done {
                    worker_id: _worker_id,
                } = &event
                {
                    active_workers -= 1;
                };

                // Add to bounded event buffer
                if app.events.len() >= MAX_EVENTS {
                    app.events.pop_front(); // Evict oldest
                }
                app.events.push_back(event);
            }
        }
    }
}

/// Renders the current screen based on the application state.
fn render(f: &mut Frame, app: &App) {
    match &app.current_screen {
        Screen::Splash => render_splash(f),
        Screen::Login => render_login(f),
        Screen::Dashboard(_state) => {
            // Update the dashboard state with the latest events
            let state =
                DashboardState::new(app.node_id, app.environment, app.start_time, &app.events);
            render_dashboard(f, &state)
        }
    }
}

/// Events emitted by prover threads.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ProverEvent {
    Message {
        worker_id: usize,
        data: String,
    },
    #[allow(unused)]
    Done {
        worker_id: usize,
    },
}

/// Spawns a new thread for the anonymous prover.
fn spawn_anonymous_prover(
    worker_id: usize,
    sender: crossbeam::channel::Sender<ProverEvent>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        // Create a new runtime for each thread
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        loop {
            rt.block_on(async {
                match prove_anonymously() {
                    Ok(_) => {
                        let now = Local::now();
                        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
                        let message = format!(
                            "✅ [{}] Proof completed successfully [Anonymous Prover {}]",
                            timestamp, worker_id
                        );
                        sender
                            .send(ProverEvent::Message {
                                worker_id,
                                data: message,
                            })
                            .unwrap();
                    }
                    Err(e) => {
                        let message = format!("Anonymous Prover {}: Error - {}", worker_id, e);
                        sender
                            .send(ProverEvent::Message {
                                worker_id,
                                data: message,
                            })
                            .unwrap();
                    }
                }
            });
        }
    })
}

/// Spawns a new thread for the prover.
fn spawn_prover(
    worker_id: usize,
    node_id: u64,
    orchestrator_client: OrchestratorClient,
    signing_key: SigningKey,
    sender: crossbeam::channel::Sender<ProverEvent>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        // Create a new runtime for each thread
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        loop {
            rt.block_on(async {
                let stwo_prover =
                    crate::prover::get_default_stwo_prover().expect("Failed to create Stwo prover");
                match authenticated_proving(
                    node_id,
                    &orchestrator_client,
                    stwo_prover,
                    signing_key.clone(),
                )
                .await
                {
                    Ok(_) => {
                        let now = Local::now();
                        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
                        let message = format!(
                            "✅ [{}] Proof completed successfully [Prover {}]",
                            timestamp, worker_id
                        );
                        sender
                            .send(ProverEvent::Message {
                                worker_id,
                                data: message,
                            })
                            .unwrap();
                    }
                    Err(e) => {
                        let message = format!("Worker {}: Error - {}", worker_id, e);
                        sender
                            .send(ProverEvent::Message {
                                worker_id,
                                data: message,
                            })
                            .unwrap();
                    }
                }
            });
        }
    })
}
