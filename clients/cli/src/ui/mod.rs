mod dashboard;
mod login;
mod splash;

use crate::environment::Environment;
use crate::orchestrator::{Orchestrator, OrchestratorClient};
use crate::prover::{authenticated_proving, prove_anonymously};
use crate::task::Task;
use crate::ui::dashboard::{DashboardState, render_dashboard};
use crate::ui::login::render_login;
use crate::ui::splash::render_splash;
use chrono::Local;
use crossterm::event::{self, Event, KeyCode};
use ed25519_dalek::SigningKey;
use ratatui::{Frame, Terminal, backend::Backend};
use std::collections::VecDeque;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::time::sleep;

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

    // Create a task master that fetches tasks from the orchestrator
    let task_queue_size = 10;
    let (task_sender, mut task_receiver) = channel::<Task>(task_queue_size);
    let task_master_handle = tokio::spawn(async move {
        let orchestrator_client = Box::new(app.orchestrator_client.clone());
        let node_id = app.node_id.expect("Node ID must be set");
        task_master(node_id, orchestrator_client, task_sender).await;
    });

    // Create workers
    let num_workers = 1; // TODO: Keep this low for now to avoid hitting rate limits.

    // Channel for sending events from workers to the main thread
    let (prover_event_sender, mut prover_event_receiver) = channel::<ProverEvent>(100);

    // One channel per worker for sending tasks.
    let mut worker_senders: Vec<Sender<Task>> = Vec::with_capacity(num_workers);

    for worker_id in 0..num_workers {
        let prover_event_sender = prover_event_sender.clone();
        let (worker_sender, mut worker_receiver) = channel::<Task>(8);
        worker_senders.push(worker_sender);
        tokio::spawn(async move {
            while let Some(task) = worker_receiver.recv().await {
                // println!("Worker {} processing {:?}", worker_id, task);
                let stwo_prover =
                    crate::prover::get_default_stwo_prover().expect("Failed to create Stwo prover");
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
    }

    // Dispatch tasks to workers. This emulates a SPMC (Single Producer, Multiple Consumer) pattern.
    tokio::spawn(async move {
        let mut next_worker = 0;
        while let Some(task) = task_receiver.recv().await {
            let target = next_worker % worker_senders.len();
            if let Err(_) = worker_senders[target].send(task).await {
                eprintln!("Worker {} has closed channel", target);
            }
            next_worker += 1;
        }
        // println!("Dispatcher exiting");
    });

    Ok(())
    // Wait for threads to finish

    // drop(sender); // Drop original sender to allow receiver to detect end-of-stream.
    // let mut active_workers = num_workers;

    // loop {
    //     match app.current_screen {
    //         Screen::Splash => {}
    //         Screen::Login => {}
    //         Screen::Dashboard(_) => {
    //             let state =
    //                 DashboardState::new(app.node_id, app.environment, app.start_time, &app.events);
    //             app.current_screen = Screen::Dashboard(state);
    //         }
    //     }
    //     terminal.draw(|f| render(f, &app.current_screen))?;
    //
    //     // Handle splash-to-login transition
    //     if let Screen::Splash = app.current_screen {
    //         if splash_start.elapsed() >= splash_duration {
    //             app.current_screen = Screen::Dashboard(DashboardState::new(
    //                 app.node_id,
    //                 app.environment,
    //                 app.start_time,
    //                 &app.events,
    //             ));
    //             continue;
    //         }
    //     }
    //
    //     // Poll for key events
    //     if event::poll(Duration::from_millis(100))? {
    //         if let Event::Key(key) = event::read()? {
    //             // Skip events that are not KeyEventKind::Press
    //             if key.kind == event::KeyEventKind::Release {
    //                 continue;
    //             }
    //
    //             // Handle exit events
    //             if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
    //                 // TODO: Close worker threads
    //                 return Ok(());
    //             }
    //
    //             match &mut app.current_screen {
    //                 Screen::Splash => {
    //                     // Any key press will skip the splash screen
    //                     if key.code != KeyCode::Esc && key.code != KeyCode::Char('q') {
    //                         app.current_screen = Screen::Dashboard(DashboardState::new(
    //                             app.node_id,
    //                             app.environment,
    //                             app.start_time,
    //                             &app.events,
    //                         ));
    //                     }
    //                 }
    //                 Screen::Login => {
    //                     todo!()
    //                     // if key.code == KeyCode::Enter {
    //                     //     app.login();
    //                     // }
    //                 }
    //                 Screen::Dashboard(_dashboard_state) => {}
    //             }
    //         }
    //     }
    // }
}

/// Renders the current screen based on the application state.
fn render(f: &mut Frame, screen: &Screen) {
    match screen {
        Screen::Splash => render_splash(f),
        Screen::Login => render_login(f),
        Screen::Dashboard(state) => {
            // Update the dashboard state with the latest events
            // let state =
            //     DashboardState::new(app.node_id, app.environment, app.start_time, &app.events);
            render_dashboard(f, &state)
        }
    }
}

/// Fetches tasks from the orchestrator and place them in the task queue.
async fn task_master(
    node_id: u64,
    orchestrator_client: Box<dyn Orchestrator>,
    sender: Sender<Task>,
) {
    println!("Task master started for node ID: {}", node_id);

    loop {
        match orchestrator_client
            .get_proof_task(&node_id.to_string())
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

        tokio::time::sleep(Duration::from_secs(1)).await;
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
fn spawn_anonymous_prover(worker_id: usize, sender: Sender<ProverEvent>) -> JoinHandle<()> {
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
                        let _ = sender
                            .send(ProverEvent::Message {
                                worker_id,
                                data: message,
                            })
                            .await;
                    }
                    Err(e) => {
                        let message = format!("Anonymous Prover {}: Error - {}", worker_id, e);
                        let _ = sender
                            .send(ProverEvent::Message {
                                worker_id,
                                data: message,
                            })
                            .await;
                    }
                }
            });
        }
    })
}

/// Spawns a new thread for the prover.
fn spawn_prover(
    worker_id: usize,
    mut task_receiver: Receiver<Task>,
    sender: Sender<ProverEvent>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            while let Some(task) = task_receiver.recv().await {
                let stwo_prover =
                    crate::prover::get_default_stwo_prover().expect("Failed to create Stwo prover");
                match authenticated_proving(task, stwo_prover).await {
                    Ok(_) => {
                        let now = Local::now();
                        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
                        let message = format!(
                            "✅ [{}] Proof completed successfully [Prover {}]",
                            timestamp, worker_id
                        );
                        let _ = sender
                            .send(ProverEvent::Message {
                                worker_id,
                                data: message,
                            })
                            .await;
                    }
                    Err(e) => {
                        let message = format!("Worker {}: Error - {}", worker_id, e);
                        let _ = sender
                            .send(ProverEvent::Message {
                                worker_id,
                                data: message,
                            })
                            .await;
                    }
                }
            }
        });
    })
}

#[cfg(test)]
mod tests {
    use crate::orchestrator::MockOrchestrator;
    use crate::task::Task;
    use crate::ui::task_master;
    use std::time::Duration;
    use tokio::sync::mpsc;

    /// Creates a mock orchestrator client that simulates fetching tasks.
    fn get_mock_orchestrator_client(num_task_requests: usize) -> MockOrchestrator {
        let tasks = (0..num_task_requests)
            .map(|i| Task::new(i.to_string(), format!("Task {}", i), vec![1, 2, 3]))
            .collect::<Vec<_>>();

        let mut call_index = 0;

        let mut mock = MockOrchestrator::new();
        mock.expect_get_proof_task()
            .times(num_task_requests)
            .returning_st(move |_| {
                println!("Mock get_proof_task called with index {}", call_index);
                // Simulate a task with dummy data
                // let task = Task::new("1".to_string(), "Test Task".to_string(), vec![1, 2, 3]);
                let task = tasks[call_index].clone();
                call_index += 1;
                Ok(task)
            });
        mock
    }

    #[tokio::test]
    // The task master should fetch and enqueue tasks from the orchestrator.
    async fn test_task_master() {
        let orchestrator_client = Box::new(get_mock_orchestrator_client(10));
        let node_id = 1003;

        let task_queue_size = 10;
        let (task_sender, mut task_receiver) = mpsc::channel::<Task>(task_queue_size);

        // Run task_master in a tokio task to stay in the same thread context
        let task_handle = tokio::spawn(async move {
            task_master(node_id, orchestrator_client, task_sender).await;
        });

        // Receive tasks
        let mut received = 0;
        for i in 0..task_queue_size {
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

        task_handle.await.unwrap();
    }
}
