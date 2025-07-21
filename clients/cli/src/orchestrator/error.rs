//! Error handling for the orchestrator module

use prost::DecodeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    Http {
        status: u16,
        message: String,
        headers: HashMap<String, String>,
    },
}

impl OrchestratorError {
    pub async fn from_response(response: reqwest::Response) -> OrchestratorError {
        let status = response.status().as_u16();

        // Capture headers before consuming the response
        let mut headers = HashMap::new();
        for (name, value) in response.headers().iter() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(name.to_string().to_lowercase(), value_str.to_string());
            }
        }

        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read response text".to_string());

        OrchestratorError::Http {
            status,
            message,
            headers,
        }
    }

    /// Get the Retry-After header value in seconds, if present
    pub fn get_retry_after_seconds(&self) -> Option<u32> {
        match self {
            Self::Http { headers, .. } => headers
                .get("retry-after")
                .and_then(|value| value.parse::<u32>().ok()),
            _ => None,
        }
    }

    pub fn to_pretty(&self) -> Option<String> {
        match self {
            Self::Http {
                status: _,
                message: msg,
                headers: _,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_retry_after_seconds() {
        let mut headers = HashMap::new();
        headers.insert("retry-after".to_string(), "120".to_string());

        let error = OrchestratorError::Http {
            status: 429,
            message: "Rate limited".to_string(),
            headers,
        };

        assert_eq!(error.get_retry_after_seconds(), Some(120));
    }

    #[test]
    fn test_get_retry_after_seconds_missing_header() {
        let error = OrchestratorError::Http {
            status: 429,
            message: "Rate limited".to_string(),
            headers: HashMap::new(),
        };

        assert_eq!(error.get_retry_after_seconds(), None);
    }

    #[test]
    fn test_get_retry_after_seconds_invalid_value() {
        let mut headers = HashMap::new();
        headers.insert("retry-after".to_string(), "invalid".to_string());

        let error = OrchestratorError::Http {
            status: 429,
            message: "Rate limited".to_string(),
            headers,
        };

        assert_eq!(error.get_retry_after_seconds(), None);
    }
}
