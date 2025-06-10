//! Nexus Orchestrator Client
//!
//! A client for the Nexus Orchestrator, allowing for proof task retrieval and submission.

use crate::environment::Environment;
use crate::nexus_orchestrator::{
    GetProofTaskRequest, GetProofTaskResponse, GetTasksRequest, GetTasksResponse, NodeType,
    RegisterNodeRequest, RegisterNodeResponse, RegisterUserRequest, SubmitProofRequest,
};
use crate::orchestrator::Orchestrator;
use crate::orchestrator::error::OrchestratorError;
use crate::system::{get_memory_info, measure_gflops};
use crate::task::Task;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use prost::Message;
use reqwest::{Client, ClientBuilder, Method};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct OrchestratorClient {
    client: Client,
    environment: Environment,
}

impl OrchestratorClient {
    pub fn new(environment: Environment) -> Self {
        Self {
            client: ClientBuilder::new()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            environment,
        }
    }

    /// Makes a request to the Nexus Orchestrator.
    ///
    /// # Arguments:
    /// * `url` - The endpoint to call, e.g., "/tasks".
    /// * `method` - The HTTP method to use, e.g., "POST" or "GET".
    /// * `request_data` - The request data to send, which must implement the `Message` trait.
    async fn make_request<T, U>(
        &self,
        url: &str,
        method: Method,
        request_data: &T,
    ) -> Result<Option<U>, OrchestratorError>
    where
        T: Message,
        U: Message + Default,
    {
        let request_bytes = request_data.encode_to_vec();
        let url = format!("{}/v3{}", self.environment.orchestrator_url(), url);
        let response = match method {
            Method::POST => {
                self.client
                    .post(&url)
                    .header("Content-Type", "application/octet-stream")
                    .body(request_bytes)
                    .send()
                    .await?
            }
            Method::GET => {
                self.client
                    .get(&url)
                    .header("Content-Type", "application/octet-stream")
                    .body(request_bytes)
                    .send()
                    .await?
            }
            _ => return Err(OrchestratorError::UnsupportedMethod(method.to_string())),
        };
        let response_bytes = response.bytes().await?;
        if response_bytes.is_empty() {
            return Ok(None);
        }

        match U::decode(response_bytes) {
            Ok(msg) => Ok(Some(msg)),
            Err(_e) => Ok(None),
        }
    }
}

#[async_trait::async_trait]
impl Orchestrator for OrchestratorClient {
    fn environment(&self) -> &Environment {
        &self.environment
    }

    /// Registers a new node with the orchestrator.
    async fn register_user(
        &self,
        user_id: &str,
        wallet_address: &str,
    ) -> Result<(), OrchestratorError> {
        let request = RegisterUserRequest {
            uuid: user_id.to_string(),
            wallet_address: wallet_address.to_string(),
        };

        self.make_request::<RegisterUserRequest, ()>("/users", Method::POST, &request)
            .await?;
        Ok(())
    }

    /// Registers a new node with the orchestrator.
    async fn register_node(&self, user_id: &str) -> Result<String, OrchestratorError> {
        let url = format!("{}/v3/nodes", self.environment.orchestrator_url());
        let request = RegisterNodeRequest {
            node_type: NodeType::CliProver as i32,
            user_id: user_id.to_string(),
        };
        let request_bytes = request.encode_to_vec();
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .body(request_bytes)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OrchestratorError::HTTPError {
                status: response.status().as_u16(),
                message: response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Failed to read response text".to_string()),
            });
        }

        let response_bytes = response.bytes().await?;
        let register_node_response: RegisterNodeResponse =
            match RegisterNodeResponse::decode(response_bytes) {
                Ok(msg) => msg,
                Err(_e) => return Err(OrchestratorError::DecodeError(_e)),
            };
        Ok(register_node_response.node_id)
    }

    async fn get_tasks(&self, node_id: &str) -> Result<Vec<Task>, OrchestratorError> {
        let url = format!("{}/v3/tasks", self.environment.orchestrator_url());
        let request = GetTasksRequest {
            node_id: node_id.to_string(),
            next_cursor: "".to_string(),
        };
        let request_bytes = request.encode_to_vec();

        let response = self
            .client
            .get(&url)
            .header("Content-Type", "application/octet-stream")
            .query(&[("nodeId", node_id.to_string())]) // Send nodeId as query param?
            .body(request_bytes)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OrchestratorError::HTTPError {
                status: response.status().as_u16(),
                message: response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Failed to read response text".to_string()),
            });
        }
        let response_bytes = response.bytes().await?;

        let get_tasks_response: GetTasksResponse = match GetTasksResponse::decode(response_bytes) {
            Ok(msg) => msg,
            Err(_e) => return Err(OrchestratorError::DecodeError(_e)),
        };

        // Convert the tasks into the Task struct
        let tasks = get_tasks_response.tasks.iter().map(Task::from).collect();
        Ok(tasks)
    }

    async fn get_proof_task(
        &self,
        node_id: &str,
        verifying_key: VerifyingKey,
    ) -> Result<Task, OrchestratorError> {
        let request = GetProofTaskRequest {
            node_id: node_id.to_string(),
            node_type: NodeType::CliProver as i32,
            ed25519_public_key: verifying_key.to_bytes().to_vec(),
        };

        let request_bytes = request.encode_to_vec();
        let url = format!("{}/v3/tasks", self.environment.orchestrator_url());

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .body(request_bytes)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OrchestratorError::HTTPError {
                status: response.status().as_u16(),
                message: response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Failed to read response text".to_string()),
            });
        }

        // Decode the response bytes
        let response_bytes = response.bytes().await?;
        let get_proof_task_response: GetProofTaskResponse =
            match GetProofTaskResponse::decode(response_bytes) {
                Ok(msg) => msg,
                Err(_e) => return Err(OrchestratorError::DecodeError(_e)),
            };
        Ok(Task::from(&get_proof_task_response))
    }

    async fn submit_proof(
        &self,
        task_id: &str,
        proof_hash: &str,
        proof: Vec<u8>,
        signing_key: SigningKey,
    ) -> Result<(), OrchestratorError> {
        let (program_memory, total_memory) = get_memory_info();
        let flops = measure_gflops();

        let signature_version = 0; // Version of the signature format
        let msg = format!(
            "version: {} | task_id: {} | proof_hash: {}",
            signature_version, task_id, proof_hash
        );
        let signature = signing_key.sign(msg.as_bytes());
        let verifying_key: VerifyingKey = signing_key.verifying_key();

        let request = SubmitProofRequest {
            task_id: task_id.to_string(),
            node_type: NodeType::CliProver as i32,
            proof_hash: proof_hash.to_string(),
            proof,
            node_telemetry: Some(crate::nexus_orchestrator::NodeTelemetry {
                flops_per_sec: Some(flops as i32),
                memory_used: Some(program_memory),
                memory_capacity: Some(total_memory),
                location: Some("US".to_string()),
            }),
            ed25519_public_key: verifying_key.to_bytes().to_vec(),
            signature: signature.to_bytes().to_vec(),
        };

        self.make_request::<SubmitProofRequest, ()>("/tasks/submit", Method::POST, &request)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
/// These are ignored by default since they require a live orchestrator to run.
mod live_orchestrator_tests {
    use crate::environment::Environment;
    use crate::orchestrator::Orchestrator;

    #[tokio::test]
    #[ignore] // This test requires a live orchestrator instance.
    /// Should register a new user with the orchestrator.
    async fn test_register_user() {
        let client = super::OrchestratorClient::new(Environment::Beta);
        // UUIDv4 for the user ID
        let user_id = uuid::Uuid::new_v4().to_string();
        let wallet_address = "0x1234567890abcdef1234567890abcabc12345678"; // Example wallet address
        match client.register_user(&user_id, wallet_address).await {
            Ok(_) => println!("User registered successfully: {}", user_id),
            Err(e) => panic!("Failed to register user: {}", e),
        }
    }

    #[tokio::test]
    #[ignore] // This test requires a live orchestrator instance.
    /// Should register a new node to an existing user.
    async fn test_register_node() {
        let client = super::OrchestratorClient::new(Environment::Beta);
        let user_id = "78db0be7-f603-4511-9576-c660f3c58395";
        match client.register_node(user_id).await {
            Ok(node_id) => println!("Node registered successfully: {}", node_id),
            Err(e) => panic!("Failed to register node: {}", e),
        }
    }

    #[tokio::test]
    #[ignore] // This test requires a live orchestrator instance.
    /// Should return a new proof task for the node.
    async fn test_get_proof_task() {
        let client = super::OrchestratorClient::new(Environment::Beta);
        let node_id = "5880437"; // Example node ID
        let signing_key = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
        let verifying_key = signing_key.verifying_key();
        let result = client.get_proof_task(node_id, verifying_key).await;
        println!("{:?}", result);
    }

    #[tokio::test]
    /// Should return the list of existing tasks for the node.
    async fn test_get_tasks() {
        let client = super::OrchestratorClient::new(Environment::Beta);
        let node_id = "5880437"; // Example node ID
        match client.get_tasks(&node_id.to_string()).await {
            Ok(tasks) => {
                println!("Retrieved {} tasks for node {}", tasks.len(), node_id);
                for task in &tasks {
                    println!("Task: {}", task);
                }
            }
            Err(e) => {
                panic!("Failed to get tasks: {}", e);
            }
        }
    }
}
