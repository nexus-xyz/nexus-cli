use std::fmt::{Display, Formatter};

/// Represents the different deployment environments available for the CLI.
#[derive(clap::ValueEnum, Debug, Clone, Default, Copy, PartialEq, Eq)]
pub enum Environment {
    /// Local development environment.
    Local,
    /// Development environment (shared).
    Dev,
    /// Staging environment for pre-production testing.
    Staging,
    /// Beta environment for limited user exposure.
    #[default]
    Beta,
}

impl Environment {
    /// Returns the orchestrator service URL associated with the environment.
    pub fn orchestrator_url(&self) -> String {
        match self {
            Environment::Local => "http://localhost:50505".to_string(),
            Environment::Dev => "https://dev.orchestrator.nexus.xyz".to_string(),
            Environment::Staging => "https://staging.orchestrator.nexus.xyz".to_string(),
            Environment::Beta => "https://beta.orchestrator.nexus.xyz".to_string(),
        }
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Local => write!(f, "Local"),
            Environment::Dev => write!(f, "Development"),
            Environment::Staging => write!(f, "Staging"),
            Environment::Beta => write!(f, "Beta"),
        }
    }
}
