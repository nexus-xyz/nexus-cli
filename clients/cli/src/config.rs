//! Application configuration.

use crate::cli_messages::{print_error, print_info, print_success};
use crate::environment::Environment;
use crate::orchestrator::Orchestrator;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// Get the path to the Nexus config file, typically located at ~/.nexus/config.json.
pub fn get_config_path() -> Result<PathBuf, std::io::Error> {
    let home_path = home::home_dir().ok_or(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Home directory not found",
    ))?;
    let config_path = home_path.join(".nexus").join("config.json");
    Ok(config_path)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    /// Environment from config file
    #[serde(default)]
    pub environment: String,

    /// User ID from config file
    #[serde(default)]
    pub user_id: String,

    /// Wallet address, resolved during `Config::resolve`
    #[serde(default)]
    pub wallet_address: String,

    /// Node ID, resolved to a valid u64 during `Config::resolve`
    #[serde(default)]
    pub node_id: String,
}

impl Config {
    /// Create Config with the given node_id.
    pub fn new(
        user_id: String,
        wallet_address: String,
        node_id: String,
        environment: Environment,
    ) -> Self {
        Config {
            user_id,
            wallet_address,
            node_id,
            environment: environment.to_string(),
        }
    }

    /// Loads configuration from a JSON file at the given path.
    pub fn load_from_file(path: &Path) -> Result<Self, std::io::Error> {
        let buf = fs::read(path)?;
        let config: Config = serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    /// Saves the configuration to a JSON file at the given path.
    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Serialization failed: {}", e),
            )
        })?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Clear the node ID configuration file.
    pub fn clear_node_config(path: &Path) -> std::io::Result<()> {
        if !path.exists() {
            println!("No config file found at {}", path.display());
            return Ok(());
        }
        if !path.ends_with("config.json") {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path must end with config.json",
            ));
        }
        fs::remove_file(path)
    }

    /// Resolves configuration and ensures node_id is available
    pub async fn resolve(
        node_id_arg: Option<u64>,
        config_path: &Path,
        orchestrator: &impl Orchestrator,
    ) -> Result<Self, Box<dyn Error>> {
        // Special case: if --node-id is provided, allow running without config file
        if let Some(node_id) = node_id_arg {
            print_success("Using provided Node ID", &format!("Node ID: {}", node_id));

            // Get the wallet address for analytics
            let wallet_address = orchestrator.get_node(&node_id.to_string()).await?;

            // Create a minimal config with the provided node_id
            let config = Config {
                user_id: "anonymous".to_string(), // Use anonymous for --node-id shortcut
                wallet_address,
                node_id: node_id.to_string(),
                environment: "".to_string(),
            };

            return Ok(config);
        }

        // For normal operation without --node-id, config file is required
        if !config_path.exists() {
            print_info(
                "Welcome to Nexus CLI!",
                "Please register your wallet address to get started: nexus-cli register-user --wallet-address <your-wallet-address>",
            );
            return Err("Configuration file not found. Please register first.".into());
        }

        // Load the config file
        let mut config = Config::load_from_file(config_path)?;

        // Resolve node_id from config file
        let resolved_node_id = match config.resolve_node_id_from_config() {
            Ok(id_from_config) => {
                print_success(
                    "Found Node ID from config file",
                    &format!("Node ID: {}", id_from_config),
                );
                id_from_config
            }
            Err(e) => {
                // The config is present but incomplete or invalid
                print_error(
                    "Your configuration is incomplete or invalid.",
                    Some("Please register your node. Start with: nexus-cli register-node"),
                );
                return Err(e);
            }
        };

        // Get the wallet address for analytics
        let wallet_address = orchestrator.get_node(&resolved_node_id.to_string()).await?;

        // Populate the config struct with the resolved values
        config.node_id = resolved_node_id.to_string();
        config.wallet_address = wallet_address;

        Ok(config)
    }

    /// Resolves node ID from the configuration file content
    fn resolve_node_id_from_config(&self) -> Result<u64, Box<dyn Error>> {
        if self.user_id.is_empty() {
            return Err("User not registered in config file.".into());
        }

        if self.node_id.is_empty() {
            print_error(
                "User registered, but no node found",
                Some("Please register a node to continue: nexus-cli register-node"),
            );
            return Err(
                "Node registration required. Please run 'nexus-cli register-node' first.".into(),
            );
        }

        match self.node_id.parse::<u64>() {
            Ok(id) => Ok(id),
            Err(_) => {
                print_error(
                    "Invalid node ID in config file",
                    Some("Please register a new node: nexus-cli register-node"),
                );
                Err(
                    "Invalid node ID in config. Please run 'nexus-cli register-node' to fix this."
                        .into(),
                )
            }
        }
    }
}
