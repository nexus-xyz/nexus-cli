//! A CLI node's local configuration file.

use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_id: String,
}

impl NodeConfig {
    /// Create a new NodeConfig with the given node_id.
    pub fn new(node_id: String) -> Self {
        NodeConfig { node_id }
    }

    /// Load the NodeConfig from a file at the given path.
    pub fn load_from_file(path: &Path) -> Result<Self, std::io::Error> {
        let buf = fs::read(path)?;
        let config: NodeConfig = serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    /// Save the NodeConfig to a file at the given path.
    ///
    /// This will overwrite any existing file at that path. If the parent directory or directories
    /// do not exist, they will be created.
    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, json)?;
        Ok(())
    }
}
