//! Headless mode execution

use super::{SessionData, messages::{print_session_starting, print_session_shutdown, print_session_exit_success}};
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


    // Trigger shutdown on Ctrl+C
    let shutdown_sender_clone = session.shutdown_sender.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            let _ = shutdown_sender_clone.send(());
        }
    });

    let mut shutdown_receiver = session.shutdown_sender.subscribe();

    // Event loop: log events to console until shutdown
    loop {
        tokio::select! {
            Some(event) = session.event_receiver.recv() => {
                println!("{}", event);
            }
            _ = shutdown_receiver.recv() => {
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