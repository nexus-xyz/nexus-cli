//! Unified messaging system for session operations

// ANSI Color Codes for session messages
pub const COLOR_INFO: &str = "\x1b[1;36m"; // Bold Cyan
pub const COLOR_SUCCESS: &str = "\x1b[1;32m"; // Bold Green
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
