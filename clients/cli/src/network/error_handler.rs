//! Centralized error handling and classification

use crate::logging::LogLevel;
use crate::orchestrator::error::OrchestratorError;

/// Centralized error handler for all network operations
#[derive(Debug, Clone)]
pub struct ErrorHandler;

impl ErrorHandler {
    pub fn new() -> Self {
        Self
    }

    /// Classify error and determine appropriate log level
    pub fn classify_error(&self, error: &OrchestratorError) -> LogLevel {
        match error {
            // Rate limiting - low priority
            OrchestratorError::Http { status, .. } if *status == 429 => LogLevel::Debug,

            // Server errors - temporary issues
            OrchestratorError::Http { status, .. } if (500..=599).contains(status) => {
                LogLevel::Warn
            }

            // Authentication errors - critical
            OrchestratorError::Http { status, .. } if *status == 401 => LogLevel::Error,
            OrchestratorError::Http { status, .. } if *status == 403 => LogLevel::Error,

            // Network issues - usually temporary
            OrchestratorError::Reqwest(_) => LogLevel::Warn,

            // Other errors
            _ => LogLevel::Warn,
        }
    }

    /// Determine if an error should trigger retry logic
    pub fn should_retry(&self, error: &OrchestratorError) -> bool {
        match error {
            // Retry on network/connection errors
            OrchestratorError::Reqwest(_) => true,
            OrchestratorError::Decode(_) => true,

            // HTTP errors - check status code
            OrchestratorError::Http { status, .. } => {
                match *status {
                    // Don't retry client errors (except rate limiting)
                    429 => false,      // Rate limiting - don't retry
                    400..=499 => true, // Other client errors - should retry
                    // Retry server errors
                    500..=599 => true,
                    // Don't retry other status codes
                    _ => false,
                }
            }
        }
    }
}
