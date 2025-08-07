//! Unified messaging system for session operations

use crate::ui::splash::LOGO_NAME;

// ANSI Color Codes for session messages
pub const COLOR_INFO: &str = "\x1b[1;36m"; // Bold Cyan
pub const COLOR_SUCCESS: &str = "\x1b[1;32m"; // Bold Green
pub const COLOR_ORANGE: &str = "\x1b[38;2;255;170;0m"; // Custom Orange
pub const COLOR_RESET: &str = "\x1b[0m";

/// Session-specific message types
#[derive(Debug, Clone)]
pub enum SessionMessage {
    /// Normal session start/shutdown messages
    Info(String),
    /// Success messages for completed operations
    Success(String),
}

impl SessionMessage {
    /// Create an info message
    pub fn info(msg: impl Into<String>) -> Self {
        Self::Info(msg.into())
    }

    /// Create a success message
    pub fn success(msg: impl Into<String>) -> Self {
        Self::Success(msg.into())
    }

    /// Print the message with appropriate formatting
    pub fn print(&self) {
        match self {
            Self::Info(msg) => {
                println!("{}[INFO]{} {}", COLOR_INFO, COLOR_RESET, msg);
            }
            Self::Success(msg) => {
                println!("{}[SUCCESS]{} {}", COLOR_SUCCESS, COLOR_RESET, msg);
            }
        }
    }
}

/// Print session startup message
pub fn print_session_starting(mode: &str, node_id: u64) {
    SessionMessage::info(format!("Starting {} mode with Node ID: {}", mode, node_id)).print();
}

/// Print session shutdown message
pub fn print_session_shutdown() {
    SessionMessage::info("Shutting down...").print();
}

/// Print session exit message
pub fn print_session_exit_success() {
    SessionMessage::success("Nexus CLI exited successfully").print();
}

/// Print orchestrator traffic notice with logo
pub fn print_orchestrator_traffic_notice() {
    // RGB: FF = 255, AA = 170, 00 = 0 (orange)
    println!("{}{}{}", COLOR_ORANGE, LOGO_NAME, COLOR_RESET);
    println!("{}We'll be back shortly!{}", COLOR_ORANGE, COLOR_RESET);
    println!(
        "The orchestrator of the prover network is under unprecedented traffic. The team has been notified. Thank you for your patience while the issue is resolved.\n"
    );
}
