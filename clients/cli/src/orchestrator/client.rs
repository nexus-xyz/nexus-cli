//! Nexus Orchestrator Client
//!
//! A client for the Nexus Orchestrator, allowing for proof task retrieval and submission.

use crate::environment::Environment;
use crate::nexus_orchestrator::{
    GetProofTaskRequest, GetProofTaskResponse, GetTasksRequest, GetTasksResponse, NodeType,
    RegisterNodeRequest, RegisterNodeResponse, RegisterUserRequest, SubmitProofRequest,
    UserResponse,
};
use crate::orchestrator::{Orchestrator, OrchestratorError};
use crate::system::{estimate_peak_gflops, get_memory_info};
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

    /// Unified helper to handle HTTP requests with protobuf
    async fn call_endpoint<Req, Resp>(
        &self,
        method: Method,
        endpoint: &str,
        request: Option<Req>,
    ) -> Result<Option<Resp>, OrchestratorError>
    where
        Req: Message,
        Resp: Message + Default,
    {
        let url = format!(
            "{}/{}",
            self.environment.orchestrator_url().trim_end_matches('/'),
            endpoint
        );

        let mut req_builder = match method {
            Method::GET => self.client.get(&url),
            Method::POST => self.client.post(&url),
            _ => {
                return Err(OrchestratorError::UnsupportedMethod {
                    method: method.to_string(),
                });
            }
        };

        req_builder = req_builder.header("Content-Type", "application/octet-stream");

        // Only add body if request is provided
        if let Some(req) = request {
            let request_bytes = req.encode_to_vec();
            req_builder = req_builder.body(request_bytes);
        }

        let response = req_builder.send().await?;

        if !response.status().is_success() {
            return Err(OrchestratorError::from_response(response).await);
        }

        let response_bytes = response.bytes().await?;

        if response_bytes.is_empty() {
            Ok(None)
        } else {
            let decoded = Resp::decode(response_bytes).map_err(OrchestratorError::Decode)?;
            Ok(Some(decoded))
        }
    }

    /// Helper for simple GET requests that expect a response
    async fn fetch_data<Resp>(&self, endpoint: &str) -> Result<Resp, OrchestratorError>
    where
        Resp: Message + Default,
    {
        self.call_endpoint::<(), Resp>(Method::GET, endpoint, None)
            .await?
            .ok_or(OrchestratorError::EmptyResponse)
    }
}

#[async_trait::async_trait]
impl Orchestrator for OrchestratorClient {
    fn environment(&self) -> &Environment {
        &self.environment
    }

    /// Get the user ID associated with a wallet address.
    async fn get_user(&self, wallet_address: &str) -> Result<String, OrchestratorError> {
        let wallet_path = urlencoding::encode(wallet_address).into_owned();
        let path = format!("v3/users/{}", wallet_path);

        let user_response: UserResponse = self.fetch_data(&path).await?;
        Ok(user_response.user_id)
    }

    /// Registers a new user with the orchestrator.
    async fn register_user(
        &self,
        user_id: &str,
        wallet_address: &str,
    ) -> Result<(), OrchestratorError> {
        let request = RegisterUserRequest {
            uuid: user_id.to_string(),
            wallet_address: wallet_address.to_string(),
        };

        self.call_endpoint::<_, ()>(Method::POST, "v3/users", Some(request))
            .await?;
        Ok(())
    }

    /// Registers a new node with the orchestrator.
    async fn register_node(&self, user_id: &str) -> Result<String, OrchestratorError> {
        let request = RegisterNodeRequest {
            node_type: NodeType::CliProver as i32,
            user_id: user_id.to_string(),
        };

        let response: RegisterNodeResponse = self
            .call_endpoint(Method::POST, "v3/nodes", Some(request))
            .await?
            .ok_or(OrchestratorError::EmptyResponse)?;
        Ok(response.node_id)
    }

    async fn get_tasks(&self, node_id: &str) -> Result<Vec<Task>, OrchestratorError> {
        let request = GetTasksRequest {
            node_id: node_id.to_string(),
            next_cursor: "".to_string(),
        };

        let response: GetTasksResponse = self
            .call_endpoint(Method::GET, "v3/tasks", Some(request))
            .await?
            .ok_or(OrchestratorError::EmptyResponse)?;
        let tasks = response.tasks.iter().map(Task::from).collect();
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

        let response: GetProofTaskResponse = self
            .call_endpoint(Method::POST, "v3/tasks", Some(request))
            .await?
            .ok_or(OrchestratorError::EmptyResponse)?;
        Ok(Task::from(&response))
    }

    async fn submit_proof(
        &self,
        task_id: &str,
        proof_hash: &str,
        proof: Vec<u8>,
        signing_key: SigningKey,
        num_provers: usize,
    ) -> Result<(), OrchestratorError> {
        let (program_memory, total_memory) = get_memory_info();
        let flops = estimate_peak_gflops(num_provers);

        let signature_version = 0;
        let msg = format!("{} | {} | {}", signature_version, task_id, proof_hash);
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

        self.call_endpoint::<_, ()>(Method::POST, "v3/tasks/submit", Some(request))
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
        let wallet_address = "0x1234567890abcdef1234567890cbaabc12345678"; // Example wallet address
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
        match result {
            Ok(task) => {
                println!("Retrieved task: {:?}", task);
            }
            Err(e) => {
                panic!("Failed to get proof task: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // This test requires a live orchestrator instance.
    /// Should return the list of existing tasks for the node.
    async fn test_get_tasks() {
        let client = super::OrchestratorClient::new(Environment::Beta);
        let node_id = "5880437"; // Example node ID
        match client.get_tasks(node_id).await {
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

    #[tokio::test]
    #[ignore] // This test requires a live orchestrator instance.
    /// Should return the user ID associated with a previously-registered wallet address.
    async fn test_get_user() {
        let client = super::OrchestratorClient::new(Environment::Beta);
        let wallet_address = "0x52908400098527886E0F7030069857D2E4169EE8";
        match client.get_user(wallet_address).await {
            Ok(user_id) => {
                println!("User ID for wallet {}: {}", wallet_address, user_id);
                assert_eq!(user_id, "e3c62f51-e566-4f9e-bccb-be9f8cb474be");
            }
            Err(e) => panic!("Failed to get user ID: {}", e),
        }
    }
}
