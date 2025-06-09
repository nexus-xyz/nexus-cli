//! Prover Task

use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Task {
    /// Orchestrator task ID
    pub task_id: String,

    /// ID of the program to be executed
    pub program_id: String,

    /// Public inputs for the task,
    pub public_inputs: Vec<u8>,

    /// Created at timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Task {
    /// Creates a new task with the given parameters.
    pub fn new(task_id: String, program_id: String, public_inputs: Vec<u8>) -> Self {
        Task {
            task_id,
            program_id,
            public_inputs,
            created_at: chrono::Utc::now(),
        }
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Task ID: {}\nProgram ID: {}\nPublic Inputs: {:?}\nCreated At: {}",
            self.task_id, self.program_id, self.public_inputs, self.created_at
        )
    }
}
