//! Error handling for the orchestrator module

use prost::DecodeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrchestratorError {
    /// Failed to read or interpret the server's response.
    #[error("Invalid response from server: {0}")]
    ResponseError(String),

    /// Failed to decode a Protobuf message from the server
    #[error("Decoding error: {0}")]
    DecodeError(#[from] DecodeError),

    /// Reqwest error, typically related to network issues or request failures.
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    /// An unsupported HTTP method was used in a request.
    #[error("Unsupported HTTP method: {0}")]
    UnsupportedMethod(String),

    /// An error occurred while processing the request.
    #[error("HTTP error with status {status}: {message}")]
    HTTPError { status: u16, message: String },
}
