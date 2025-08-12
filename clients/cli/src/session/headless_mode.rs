//! Headless mode execution

use super::{
    SessionData,
    messages::{print_session_exit_success, print_session_shutdown, print_session_starting},
};
use crate::print_cmd_info;
use crate::version::checker::check_for_new_version;
use std::error::Error;

/// Runs the application in headless mode
///
/// This function handles:
/// 1. Console event logging
/// 2. Ctrl+C shutdown handling
/// 3. Event loop management
///
/// # Arguments
/// * `session` - Session data from setup
///
/// # Returns
/// * `Ok(())` - Headless mode completed successfully
/// * `Err` - Headless mode failed
pub async fn run_headless_mode(mut session: SessionData) -> Result<(), Box<dyn Error>> {
    // Print session start message
    print_session_starting("headless", session.node_id);

    // Check for new version and inform user
    let current_version = env!("CARGO_PKG_VERSION");

    // First check constraint violations
    if let Some(message) = check_for_new_version(current_version).await {
        // If no constraints violated, check for newer versions available
        print_cmd_info!("Version check", "{}", message);
    }

    // Trigger shutdown on Ctrl+C
    let shutdown_sender_clone = session.shutdown_sender.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            let _ = shutdown_sender_clone.send(());
        }
    });

    let mut shutdown_receiver = session.shutdown_sender.subscribe();
    let mut max_tasks_shutdown_receiver = session.max_tasks_shutdown_sender.subscribe();

    // Event loop: log events to console until shutdown
    loop {
        tokio::select! {
            Some(event) = session.event_receiver.recv() => {
                println!("{}", event);
            }
            _ = shutdown_receiver.recv() => {
                break;
            }
            _ = max_tasks_shutdown_receiver.recv() => {
                break;
            }
        }
    }

    // Wait for workers to finish
    print_session_shutdown();
    for handle in session.join_handles {
        let _ = handle.await;
    }
    print_session_exit_success();

    Ok(())
}
