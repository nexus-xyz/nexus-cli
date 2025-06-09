//! Prover Task
//!
//! This abstracts over the two "task" types used in the Nexus Orchestrator:
//! * Task (Returned by GetTasks)
//! * GetProofTaskResponse.

use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Task {
    /// Orchestrator task ID
    pub task_id: String,

    /// ID of the program to be executed
    pub program_id: String,

    /// Public inputs for the task,
    pub public_inputs: Vec<u8>,
}

impl Task {
    /// Creates a new task with the given parameters.
    #[allow(unused)]
    pub fn new(task_id: String, program_id: String, public_inputs: Vec<u8>) -> Self {
        Task {
            task_id,
            program_id,
            public_inputs,
        }
    }
}

// Display
impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Task ID: {}, Program ID: {}, Public Inputs: {:?}",
            self.task_id, self.program_id, self.public_inputs
        )
    }
}

// From Task
impl From<&crate::nexus_orchestrator::Task> for Task {
    fn from(task: &crate::nexus_orchestrator::Task) -> Self {
        Task {
            task_id: task.task_id.clone(),
            program_id: task.program_id.clone(),
            public_inputs: task.public_inputs.clone(),
        }
    }
}

// From GetProofTaskResponse
impl From<&crate::nexus_orchestrator::GetProofTaskResponse> for Task {
    fn from(response: &crate::nexus_orchestrator::GetProofTaskResponse) -> Self {
        Task {
            task_id: response.task_id.clone(),
            program_id: response.program_id.clone(),
            public_inputs: response.public_inputs.clone(),
        }
    }
}
