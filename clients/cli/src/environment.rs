use std::fmt::{Display, Formatter};

// The Environment enum represents different networks the CLI can connect to.
#[derive(Debug, Clone)]
pub enum Environment {
    Local,
    Dev,
    Staging,
    Beta,
}

impl Environment {
    pub fn orchestrator_url(&self) -> String {
        match self {
            Environment::Local => "http://localhost:50505".to_string(),
            Environment::Dev => "https://dev.orchestrator.nexus.xyz".to_string(),
            Environment::Staging => "https://staging.orchestrator.nexus.xyz".to_string(),
            Environment::Beta => "https://beta.orchestrator.nexus.xyz".to_string(),
        }
    }

    pub fn from_args(env: Option<&crate::Environment>) -> Self {
        match env {
            Some(crate::Environment::Local) => Environment::Local,
            Some(crate::Environment::Dev) => Environment::Dev,
            Some(crate::Environment::Staging) => Environment::Staging,
            Some(crate::Environment::Beta) => Environment::Beta,
            None => Environment::Local, // Default
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
