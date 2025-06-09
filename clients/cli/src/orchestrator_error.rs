//! Error handling for the orchestrator module

use prost::DecodeError;
use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrchestratorError {
    /// The client was unable to establish a connection to the orchestrator service.
    #[error("Connection failed: {0}")]
    ConnectionError(String),

    /// An error in reading or interpreting the server's response.
    #[error("Invalid response from server: {0}")]
    ResponseError(String),

    /// A failure to decode a Protobuf message from the server
    #[error("Decoding error: {0}")]
    DecodeError(#[from] DecodeError),

    /// An HTTP-level error with a specific status code and message.
    #[error("HTTP {status}: {message}")]
    HttpError { status: StatusCode, message: String },

    /// Indicates that a server response was expected but none was received.
    #[error("Missing expected response")]
    MissingResponse,

    /// Reqwest error, typically related to network issues or request failures.
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    /// An unsupported HTTP method was used in a request.
    #[error("Unsupported HTTP method: {0}")]
    UnsupportedMethod(String),
}
