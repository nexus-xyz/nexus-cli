use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

/// Represents the different deployment environments available for the CLI.
#[derive(Clone, Default, Copy, PartialEq, Eq)]
pub enum Environment {
    /// Local development environment.
    Local,
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
            Environment::Staging => "https://staging.orchestrator.nexus.xyz".to_string(),
            Environment::Beta => "https://beta.orchestrator.nexus.xyz".to_string(),
        }
    }
}

impl FromStr for Environment {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "staging" => Ok(Environment::Staging),
            "beta" => Ok(Environment::Beta),
            _ => Err(()),
        }
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Local => write!(f, "Local"),
            Environment::Staging => write!(f, "Staging"),
            Environment::Beta => write!(f, "Beta"),
        }
    }
}

impl Debug for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Environment::{}, URL: {}", self, self.orchestrator_url())
    }
}
