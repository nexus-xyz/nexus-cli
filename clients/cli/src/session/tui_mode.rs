//! TUI mode execution

use super::{SessionData, messages::{print_session_starting, print_session_shutdown, print_session_exit_success}};
use crate::orchestrator::Orchestrator;
use crate::ui;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{error::Error, io};

/// Runs the application in TUI mode
///
/// This function handles:
/// 1. Terminal setup and cleanup
/// 2. UI application initialization and execution
/// 3. Proper shutdown handling
///
/// # Arguments
/// * `session` - Session data from setup
/// * `no_background_color` - Whether to disable background colors
///
/// # Returns
/// * `Ok(())` - TUI mode completed successfully
/// * `Err` - TUI mode failed
pub async fn run_tui_mode(
    session: SessionData,
    no_background_color: bool,
) -> Result<(), Box<dyn Error>> {
    // Print session start message
    print_session_starting("TUI", session.node_id);

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Initialize the terminal with Crossterm backend
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create the application and run it
    let app = ui::App::new(
        Some(session.node_id),
        session.orchestrator.environment().clone(),
        session.event_receiver,
        session.shutdown_sender.clone(),
        no_background_color,
        session.num_workers,
    );

    let result = ui::run(&mut terminal, app).await;

    // Clean up the terminal after running the application
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Handle the result
    result?;

    // Wait for workers to finish
    print_session_shutdown();
    for handle in session.join_handles {
        let _ = handle.await;
    }
    print_session_exit_success();

    Ok(())
}