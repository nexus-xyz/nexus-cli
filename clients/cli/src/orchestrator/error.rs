//! Error handling for the orchestrator module

use prost::DecodeError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[allow(non_snake_case)] // used for json parsing
#[derive(Serialize, Deserialize)]
struct RawError {
    name: String,
    message: String,
    httpCode: u16,
}

#[derive(Debug, Error)]
pub enum OrchestratorError {
    /// Failed to decode a Protobuf message from the server
    #[error("Decoding error: {0}")]
    Decode(#[from] DecodeError),

    /// Reqwest error, typically related to network issues or request failures.
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// An error occurred while processing the request.
    #[error("HTTP error with status {status}: {message}")]
    Http { status: u16, message: String },
}

impl OrchestratorError {
    pub async fn from_response(response: reqwest::Response) -> OrchestratorError {
        let status = response.status().as_u16();
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read response text".to_string());

        OrchestratorError::Http { status, message }
    }

    pub fn to_pretty(&self) -> Option<String> {
        match self {
            Self::Http {
                status: _,
                message: msg,
            } => {
                if let Ok(parsed) = serde_json::from_str::<RawError>(msg) {
                    if let Ok(stringified) = serde_json::to_string_pretty(&parsed) {
                        return Some(stringified);
                    }
                }

                None
            }
            _ => None,
        }
    }
}
