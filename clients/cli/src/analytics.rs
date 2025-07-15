use crate::environment::Environment;
use crate::system::{estimate_peak_gflops, measure_gflops, num_cores};
use chrono::Datelike;
use chrono::Timelike;
use reqwest::header::ACCEPT;
use serde_json::{Value, json};
use std::{
    env,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

/// Metrics context for tracking proof-related statistics
#[derive(Debug)]
pub struct ProofMetrics {
    invalid_proof_count: AtomicU64,
}

impl ProofMetrics {
    /// Create a new metrics context
    pub fn new() -> Self {
        Self {
            invalid_proof_count: AtomicU64::new(0),
        }
    }

    /// Get the current invalid proof count
    pub fn get_invalid_proof_count(&self) -> u64 {
        self.invalid_proof_count.load(Ordering::Relaxed)
    }

    /// Increment the invalid proof count and return the new count
    pub fn increment_invalid_proof_count(&self) -> u64 {
        self.invalid_proof_count.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Reset the invalid proof count (useful for testing)
    #[cfg(test)]
    pub fn reset(&self) {
        self.invalid_proof_count.store(0, Ordering::Relaxed);
    }
}

impl Default for ProofMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TrackError {
    #[error("System time error: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),

    #[error("event_properties is not a valid JSON object")]
    InvalidEventProperties,

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Non-successful response: {status} - {body}")]
    FailedResponse {
        status: reqwest::StatusCode,
        body: String,
    },
}

pub const STAGING_MEASUREMENT_ID: &str = "G-T0M0Q3V6WN";
pub const BETA_MEASUREMENT_ID: &str = "G-GLH0GMEEFH";
pub const STAGING_API_SECRET: &str = "OI7H53soRMSDWfJf1ittHQ";
pub const BETA_API_SECRET: &str = "3wxu8FjVSPqOlxSsZEnBOw";

pub fn analytics_id(environment: &Environment) -> String {
    match environment {
        Environment::Staging => STAGING_MEASUREMENT_ID.to_string(),
        Environment::Beta => BETA_MEASUREMENT_ID.to_string(),
        Environment::Local => String::new(),
    }
}

pub fn analytics_api_key(environment: &Environment) -> String {
    match environment {
        Environment::Staging => STAGING_API_SECRET.to_string(),
        Environment::Beta => BETA_API_SECRET.to_string(),
        Environment::Local => String::new(),
    }
}

/// Track an event with the Firebase Measurement Protocol
///
/// # Arguments
/// * `event_name` - The name of the event to track.
/// * `event_properties` - A JSON object containing properties of the event.
/// * `environment` - The environment in which the application is running.
/// * `client_id` - A unique identifier for the client, typically a UUID or similar.
pub async fn track(
    event_names: Vec<String>,
    event_properties: Value,
    environment: &Environment,
    client_id: String,
) -> Result<(), TrackError> {
    let analytics_id = analytics_id(environment);
    let analytics_api_key = analytics_api_key(environment);
    if analytics_id.is_empty() {
        return Ok(());
    }
    let local_now = chrono::offset::Local::now();

    // For tracking events, we use the Firebase Measurement Protocol
    // Firebase is mostly designed for mobile and web apps, but for our use case of a CLI,
    // we can use the Measurement Protocol to track events by POST to a URL.
    // The only thing that may be unexpected is that the URL we use includes a firebase key

    // Firebase format for properties for Measurement protocol:
    // https://developers.google.com/analytics/devguides/collection/protocol/ga4/reference?client_type=firebase#payload
    // https://developers.google.com/analytics/devguides/collection/protocol/ga4/reference?client_type=firebase#payload_query_parameters

    let system_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let timezone = iana_time_zone::get_timezone().ok().map_or_else(
        || String::from("UTC"), // fallback to UTC
        |tz| tz,
    );

    let mut properties = json!({
        "time": system_time,
        "platform": "CLI",
        "os": env::consts::OS,
        "os_version": env::consts::OS,  // We could get more specific version if needed
        "app_version": env!("CARGO_PKG_VERSION"),
        "timezone": timezone,
        "local_hour": local_now.hour(),
        "day_of_week": local_now.weekday().number_from_monday(),
        "event_id": system_time,
        "measured_flops": measure_gflops(),
        "num_cores": num_cores(),
        "peak_flops": estimate_peak_gflops(num_cores()),
        "invalid_proof_count": 0,
    });

    // Add event properties to the properties JSON
    // This is done by iterating over the key-value pairs in the event_properties JSON object
    // but checking that it is a valid JSON object first
    if let Some(obj) = event_properties.as_object() {
        for (k, v) in obj {
            properties[k] = v.clone();
        }
    } else {
        return Err(TrackError::InvalidEventProperties);
    }

    // Format for events
    let body = json!({
        "client_id": client_id,
        "events": event_names.iter().map(|event_name| {
            json!({
                "name": event_name,
                "params": properties
            })
        }).collect::<Vec<_>>(),
    });

    let client = reqwest::Client::new();
    let url = format!(
        "https://www.google-analytics.com/mp/collect?measurement_id={}&api_secret={}",
        analytics_id, analytics_api_key
    );

    let response = client
        .post(&url)
        .json(&body)
        .header(ACCEPT, "application/json")
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let body_text = response.text().await?;
        return Err(TrackError::FailedResponse {
            status,
            body: body_text,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_metrics_creation() {
        let metrics = ProofMetrics::new();
        assert_eq!(metrics.get_invalid_proof_count(), 0);
    }

    #[test]
    fn test_proof_metrics_increment() {
        let metrics = ProofMetrics::new();

        let count1 = metrics.increment_invalid_proof_count();
        assert_eq!(count1, 1);
        assert_eq!(metrics.get_invalid_proof_count(), 1);

        let count2 = metrics.increment_invalid_proof_count();
        assert_eq!(count2, 2);
        assert_eq!(metrics.get_invalid_proof_count(), 2);
    }

    #[test]
    fn test_proof_metrics_reset() {
        let metrics = ProofMetrics::new();

        metrics.increment_invalid_proof_count();
        metrics.increment_invalid_proof_count();
        assert_eq!(metrics.get_invalid_proof_count(), 2);

        metrics.reset();
        assert_eq!(metrics.get_invalid_proof_count(), 0);
    }

    #[test]
    fn test_proof_metrics_default() {
        let metrics = ProofMetrics::default();
        assert_eq!(metrics.get_invalid_proof_count(), 0);
    }
}
