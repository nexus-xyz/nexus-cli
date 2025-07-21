use crate::environment::Environment;
use crate::system::{estimate_peak_gflops, measure_gflops, num_cores};
use crate::task::Task;
use chrono::Datelike;
use chrono::Timelike;
use reqwest::header::ACCEPT;
use serde_json::{Value, json};
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

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

pub const PRODUCTION_MEASUREMENT_ID: &str = "G-GLH0GMEEFH";
pub const PRODUCTION_API_SECRET: &str = "3wxu8FjVSPqOlxSsZEnBOw";

pub fn analytics_id(environment: &Environment) -> String {
    match environment {
        Environment::Production => PRODUCTION_MEASUREMENT_ID.to_string(),
        Environment::Custom { .. } => String::new(), // Disable analytics for custom environments
    }
}

pub fn analytics_api_key(environment: &Environment) -> String {
    match environment {
        Environment::Production => PRODUCTION_API_SECRET.to_string(),
        Environment::Custom { .. } => String::new(), // Disable analytics for custom environments
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

/// Track analytics for getting a task from orchestrator (non-blocking)
pub async fn track_got_task(task: crate::task::Task, environment: Environment, client_id: String) {
    let analytics_data = json!({
        "program_name": task.program_id,
        "task_id": task.task_id,
    });

    let _ = track(
        vec!["cli_got_task".to_string(), "got_task".to_string()],
        analytics_data,
        &environment,
        client_id,
    )
    .await;
    // TODO: Catch errors and log them
}

/// Track analytics for proof verification failure (non-blocking)
pub async fn track_verification_failed(
    task: crate::task::Task,
    error: String,
    environment: Environment,
    client_id: String,
) {
    let analytics_data = json!({
        "program_name": task.program_id,
        "task_id": task.task_id,
        "error": error,
    });

    let _ = track(
        vec![
            "cli_local_verification_failed".to_string(),
            "local_verification_failed".to_string(),
        ],
        analytics_data,
        &environment,
        client_id,
    )
    .await;
    // TODO: Catch errors and log them
}

/// Track analytics for proof submission error (non-blocking)
pub async fn track_proof_submission_error(
    task: crate::task::Task,
    error: String,
    status_code: Option<u16>,
    environment: Environment,
    client_id: String,
) {
    let mut analytics_data = json!({
        "program_name": task.program_id,
        "task_id": task.task_id,
        "error": error,
    });

    if let Some(status) = status_code {
        analytics_data["status_code"] = json!(status);
    }

    let _ = track(
        vec![
            "cli_proof_submission_error".to_string(),
            "proof_submission_error".to_string(),
        ],
        analytics_data,
        &environment,
        client_id,
    )
    .await;
    // TODO: Catch errors and log them
}

/// Track analytics for proof acceptance (non-blocking)
pub async fn track_proof_accepted(
    task: crate::task::Task,
    environment: Environment,
    client_id: String,
) {
    let analytics_data = json!({
        "program_name": task.program_id,
        "task_id": task.task_id,
    });

    let _ = track(
        vec![
            "cli_proof_accepted".to_string(),
            "proof_accepted".to_string(),
        ],
        analytics_data,
        &environment,
        client_id,
    )
    .await;
    // TODO: Catch errors and log them
}

/// Track analytics for proof submission success (non-blocking)
pub async fn track_proof_submission_success(
    task: crate::task::Task,
    environment: Environment,
    client_id: String,
) {
    let analytics_data = json!({
        "program_name": task.program_id,
        "task_id": task.task_id,
    });

    let _ = track(
        vec![
            "cli_proof_submission_success".to_string(),
            "proof_submission_success".to_string(),
        ],
        analytics_data,
        &environment,
        client_id,
    )
    .await;
    // TODO: Catch errors and log them
}

/// Track analytics for authenticated proof (non-blocking)
pub async fn track_authenticated_proof_analytics(
    task: Task,
    environment: Environment,
    client_id: String,
) {
    let analytics_data = match task.program_id.as_str() {
        "fast-fib" => {
            // For fast-fib, extract the input from task public_inputs
            let input = if !task.public_inputs.is_empty() {
                task.public_inputs[0] as u32
            } else {
                0
            };
            json!({
                "program_name": "fast-fib",
                "public_input": input,
                "task_id": task.task_id,
            })
        }
        "fib_input_initial" => {
            // For fib_input_initial, extract the triple inputs
            let inputs = if task.public_inputs.len() >= 12 {
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&task.public_inputs[0..4]);
                let n = u32::from_le_bytes(bytes);
                bytes.copy_from_slice(&task.public_inputs[4..8]);
                let init_a = u32::from_le_bytes(bytes);
                bytes.copy_from_slice(&task.public_inputs[8..12]);
                let init_b = u32::from_le_bytes(bytes);
                (n, init_a, init_b)
            } else {
                (0, 0, 0)
            };
            json!({
                "program_name": "fib_input_initial",
                "public_input": inputs.0,
                "public_input_2": inputs.1,
                "public_input_3": inputs.2,
                "task_id": task.task_id,
            })
        }
        _ => {
            json!({
                "program_name": task.program_id,
                "task_id": task.task_id,
            })
        }
    };

    let _ = track(
        vec!["cli_proof_node_v4".to_string(), "proof_node".to_string()],
        analytics_data,
        &environment,
        client_id,
    )
    .await;
    // TODO: Catch errors and log them
}

/// Track analytics for anonymous proof (non-blocking)
pub async fn track_anonymous_proof_analytics(environment: Environment, client_id: String) {
    // Anonymous proofs use hardcoded input: (n=9, init_a=1, init_b=1)
    let public_input = (9, 1, 1);

    let _ = track(
        vec!["cli_proof_anon_v3".to_string()],
        json!({
            "program_name": "fib_input_initial",
            "public_input": public_input.0,
            "public_input_2": public_input.1,
            "public_input_3": public_input.2,
        }),
        &environment,
        client_id,
    )
    .await;
    // TODO: Catch errors and log them
}
