mod dashboard;
mod login;
mod splash;

use crate::environment::Environment;
use crate::orchestrator::{Orchestrator, OrchestratorClient};
use crate::prover::authenticated_proving;
use crate::task::Task;
use crate::ui::dashboard::{DashboardState, render_dashboard};
use crate::ui::login::render_login;
use crate::ui::splash::render_splash;
use chrono::Local;
use crossterm::event::{self, Event, KeyCode};
use ed25519_dalek::{SigningKey, VerifyingKey};
use ratatui::{Frame, Terminal, backend::Backend};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::task::JoinHandle;

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

    // TODO: prove anonymously if node_id is None

    // Create a task master that fetches tasks from the orchestrator
    let task_queue_size = 10;
    let (task_sender, task_receiver) = channel::<Task>(task_queue_size);
    let verifying_key = app.signing_key.verifying_key();

    // Spawn a Tokio task to fetch proving tasks from the orchestrator.
    let _fetch_prover_tasks_handle = tokio::spawn(async move {
        let orchestrator_client = Box::new(app.orchestrator_client.clone());
        let node_id = app.node_id.expect("Node ID must be set");
        fetch_prover_tasks(node_id, verifying_key, orchestrator_client, task_sender).await;
    });

    // Channel for sending events from workers to the main thread
    let (prover_event_sender, mut prover_event_receiver) = channel::<ProverEvent>(100);
    let num_workers = 3; // TODO: Keep this low for now to avoid hitting rate limits.
    let (worker_senders, _worker_handles) = start_workers(num_workers, prover_event_sender.clone());

    // Dispatch tasks to workers. This emulates a SPMC (Single Producer, Multiple Consumer) pattern.
    let _dispatcher_handle = start_dispatcher(task_receiver, worker_senders);

    // UI event loop
    loop {
        // Drain prover events from the async channel into app.events
        while let Ok(event) = prover_event_receiver.try_recv() {
            if app.events.len() >= MAX_EVENTS {
                app.events.pop_front();
            }
            app.events.push_back(event);
        }

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

/// Fetches tasks from the orchestrator and place them in the task queue.
async fn fetch_prover_tasks(
    node_id: u64,
    verifying_key: VerifyingKey,
    orchestrator_client: Box<dyn Orchestrator>,
    sender: Sender<Task>,
) {
    loop {
        match orchestrator_client
            .get_proof_task(&node_id.to_string(), verifying_key)
            .await
        {
            Ok(task) => {
                if sender.send(task).await.is_err() {
                    println!("sender.send() failed, task queue is closed");
                    return;
                }
            }
            Err(e) => {
                println!("Failed to fetch task: {}", e);
            }
        }
        // Be polite to the orchestrator and wait a bit before fetching the next task.
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

/// Events emitted by prover (worker) threads.
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

/// Spawns a set of worker tasks that receive tasks and send prover events.
///
/// # Arguments
/// * `num_workers` - The number of worker tasks to spawn.
/// * `prover_event_sender` - The channel to send prover events to the main thread.
///
/// # Returns
/// A tuple containing:
/// * A vector of `Sender<Task>` for each worker, allowing tasks to be sent to them.
/// * A vector of `JoinHandle<()>` for each worker, allowing the main thread to await their completion.
pub fn start_workers(
    num_workers: usize,
    prover_event_sender: Sender<ProverEvent>,
) -> (Vec<Sender<Task>>, Vec<JoinHandle<()>>) {
    let mut senders = Vec::with_capacity(num_workers);
    let mut handles = Vec::with_capacity(num_workers);

    for worker_id in 0..num_workers {
        let (task_sender, mut task_receiver) = tokio::sync::mpsc::channel::<Task>(8);
        let prover_event_sender = prover_event_sender.clone();

        let handle = tokio::spawn(async move {
            while let Some(task) = task_receiver.recv().await {
                let stwo_prover =
                    crate::prover::get_default_stwo_prover().expect("Failed to create prover");
                match authenticated_proving(task, stwo_prover).await {
                    Ok(_) => {
                        let now = Local::now();
                        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
                        let message = format!(
                            "✅ [{}] Proof completed successfully [Prover {}]",
                            timestamp, worker_id
                        );
                        let _ = prover_event_sender
                            .send(ProverEvent::Message {
                                worker_id,
                                data: message,
                            })
                            .await;
                    }
                    Err(e) => {
                        let message = format!("Worker {}: Error - {}", worker_id, e);
                        let _ = prover_event_sender
                            .send(ProverEvent::Message {
                                worker_id,
                                data: message,
                            })
                            .await;
                    }
                }
            }
        });

        senders.push(task_sender);
        handles.push(handle);
    }

    (senders, handles)
}

/// Spawns a dispatcher that forwards tasks to available workers in round-robin fashion.
pub fn start_dispatcher(
    mut task_receiver: Receiver<Task>,
    worker_senders: Vec<Sender<Task>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut next_worker = 0;
        while let Some(task) = task_receiver.recv().await {
            let target = next_worker % worker_senders.len();
            if (worker_senders[target].send(task).await).is_err() {
                eprintln!("Dispatcher: worker {} channel closed", target);
            }
            next_worker += 1;
        }
    })
}

// /// Spawns a new thread for the anonymous prover.
// fn spawn_anonymous_prover(worker_id: usize, sender: Sender<ProverEvent>) -> JoinHandle<()> {
//     thread::spawn(move || {
//         // Create a new runtime for each thread
//         let rt = Runtime::new().expect("Failed to create Tokio runtime");
//         loop {
//             rt.block_on(async {
//                 match prove_anonymously() {
//                     Ok(_) => {
//                         let now = Local::now();
//                         let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
//                         let message = format!(
//                             "✅ [{}] Proof completed successfully [Anonymous Prover {}]",
//                             timestamp, worker_id
//                         );
//                         let _ = sender
//                             .send(ProverEvent::Message {
//                                 worker_id,
//                                 data: message,
//                             })
//                             .await;
//                     }
//                     Err(e) => {
//                         let message = format!("Anonymous Prover {}: Error - {}", worker_id, e);
//                         let _ = sender
//                             .send(ProverEvent::Message {
//                                 worker_id,
//                                 data: message,
//                             })
//                             .await;
//                     }
//                 }
//             });
//         }
//     })
// }

#[cfg(test)]
mod tests {
    use crate::orchestrator::MockOrchestrator;
    use crate::task::Task;
    use crate::ui::fetch_prover_tasks;
    use std::time::Duration;
    use tokio::sync::mpsc;

    /// Creates a mock orchestrator client that simulates fetching tasks.
    fn get_mock_orchestrator_client() -> MockOrchestrator {
        let mut i = 0;
        let mut mock = MockOrchestrator::new();
        mock.expect_get_proof_task().returning_st(move |_, _| {
            // Simulate a task with dummy data
            let task = Task::new(i.to_string(), format!("Task {}", i), vec![1, 2, 3]);
            i += 1;
            Ok(task)
        });
        mock
    }

    #[tokio::test]
    // The task master should fetch and enqueue tasks from the orchestrator.
    async fn test_task_master() {
        let orchestrator_client = Box::new(get_mock_orchestrator_client());
        let signer_key = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        let verifying_key = signer_key.verifying_key();
        let node_id = 1234;

        let task_queue_size = 10;
        let (task_sender, mut task_receiver) = mpsc::channel::<Task>(task_queue_size);

        // Run task_master in a tokio task to stay in the same thread context
        let task_master_handle = tokio::spawn(async move {
            fetch_prover_tasks(node_id, verifying_key, orchestrator_client, task_sender).await;
        });

        // Receive tasks
        let mut received = 0;
        for _i in 0..task_queue_size {
            match tokio::time::timeout(Duration::from_secs(2), task_receiver.recv()).await {
                Ok(Some(task)) => {
                    println!("Received task {}: {:?}", received, task);
                    received += 1;
                }
                Ok(None) => {
                    eprintln!("Channel closed unexpectedly");
                    break;
                }
                Err(_) => {
                    eprintln!("Timed out waiting for task {}", received);
                    break;
                }
            }
        }

        task_master_handle.abort();
    }
}
