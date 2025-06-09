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
            Method::GET => self.client.get(&url).send().await?,
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
        let request = RegisterNodeRequest {
            node_type: NodeType::CliProver as i32,
            user_id: user_id.to_string(),
        };

        match self
            .make_request::<RegisterNodeRequest, RegisterNodeResponse>(
                "/nodes",
                Method::POST,
                &request,
            )
            .await?
        {
            Some(response) => Ok(response.node_id),
            None => Err(OrchestratorError::ResponseError(
                "No node ID received".to_string(),
            )),
        }
    }

    async fn get_tasks(&self, node_id: &str) -> Result<Vec<Task>, OrchestratorError> {
        let request = GetTasksRequest {
            node_id: node_id.to_string(),
            next_cursor: "".to_string(),
        };
        let url = format!("/tasks/{}", node_id);
        match self
            .make_request::<GetTasksRequest, GetTasksResponse>(&url, Method::POST, &request)
            .await?
        {
            Some(response) => {
                let tasks = response.tasks.iter().map(Task::from).collect();
                Ok(tasks)
            }
            None => Err(OrchestratorError::ResponseError(
                "No tasks found".to_string(),
            )),
        }
    }

    async fn get_proof_task(&self, node_id: &str) -> Result<Task, OrchestratorError> {
        let request = GetProofTaskRequest {
            node_id: node_id.to_string(),
            node_type: NodeType::CliProver as i32,
        };

        match self
            .make_request::<GetProofTaskRequest, GetProofTaskResponse>(
                "/tasks",
                Method::POST,
                &request,
            )
            .await?
        {
            Some(get_proof_task_response) => Ok(Task::from(&get_proof_task_response)),
            None => Err(OrchestratorError::ResponseError(
                "No task found".to_string(),
            )),
        }
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
