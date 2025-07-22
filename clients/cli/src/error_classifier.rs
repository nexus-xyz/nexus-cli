use crate::orchestrator::error::OrchestratorError;
use crate::prover::ProverError;
use log::LevelFilter;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ErrorClassifier;

impl ErrorClassifier {
    pub fn new() -> Self {
        Self
    }

    pub fn classify_fetch_error(&self, error: &OrchestratorError) -> LogLevel {
        match error {
            // Non-critical: Temporary server issues
            OrchestratorError::Http { status, .. } if *status == 429 => LogLevel::Debug,
            OrchestratorError::Http { status, .. } if (500..=599).contains(status) => {
                LogLevel::Warn
            }

            // Critical: Auth, malformed responses
            OrchestratorError::Http { status, .. } if *status == 401 => LogLevel::Error,
            OrchestratorError::Http { status, .. } if *status == 403 => LogLevel::Error,

            // Network issues - usually temporary
            _ => LogLevel::Warn,
        }
    }

    pub fn classify_worker_error(&self, error: &ProverError) -> LogLevel {
        match error {
            // Temporary resource issues
            ProverError::Stwo(msg) if msg.contains("memory") => LogLevel::Warn,
            ProverError::Stwo(msg) if msg.contains("timeout") => LogLevel::Warn,
            ProverError::Stwo(msg) if msg.contains("resource") => LogLevel::Warn,

            // Critical: Code/logic errors
            ProverError::MalformedTask(_) => LogLevel::Error,
            ProverError::GuestProgram(_) => LogLevel::Error,
            ProverError::Serialization(_) => LogLevel::Error,

            // Default to warning for other Stwo errors
            ProverError::Stwo(_) => LogLevel::Warn,
            // Version requirement errors are critical - user needs to upgrade
        }
    }
}

impl Default for ErrorClassifier {
    fn default() -> Self {
        Self::new()
    }
}
