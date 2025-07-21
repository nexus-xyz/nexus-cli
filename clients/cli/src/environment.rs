use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

/// Represents the different deployment environments available for the CLI.
#[derive(Clone, PartialEq, Eq, Default)]
pub enum Environment {
    /// Production environment.
    #[default]
    Production,
    /// Custom environment with a specific orchestrator URL.
    Custom { orchestrator_url: String },
}

impl Environment {
    /// Returns the orchestrator service URL associated with the environment.
    pub fn orchestrator_url(&self) -> &str {
        match self {
            Environment::Production => "https://production.orchestrator.nexus.xyz",
            Environment::Custom { orchestrator_url } => orchestrator_url,
        }
    }
}

impl FromStr for Environment {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "production" => Ok(Environment::Production),
            _ => Err(()),
        }
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Production => write!(f, "Production"),
            Environment::Custom { orchestrator_url } => write!(f, "Custom({})", orchestrator_url),
        }
    }
}

impl Debug for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Environment::{}, URL: {}", self, self.orchestrator_url())
    }
}
