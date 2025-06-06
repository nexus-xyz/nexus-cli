// Copyright (c) 2024 Nexus. All rights reserved.

mod analytics;
mod config;
mod environment;
mod keys;
#[path = "proto/nexus.orchestrator.rs"]
mod nexus_orchestrator;
mod orchestrator_client;
mod prover;
pub mod system;
mod ui;

use crate::config::{Config, get_config_path};
use crate::environment::Environment;
use crate::orchestrator_client::OrchestratorClient;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ed25519_dalek::SigningKey;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{error::Error, io};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// Command-line arguments
struct Args {
    /// Command to execute
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the prover
    Start {
        /// Node ID
        #[arg(long, value_name = "NODE_ID")]
        node_id: Option<u64>,

        /// Maximum number of threads to use for proving.
        #[arg(long)]
        max_threads: Option<u32>,
    },
    /// Register a new user
    RegisterUser {
        /// User's public Ethereum wallet address. 42-character hex string starting with '0x'
        #[arg(long, value_name = "WALLET_ADDRESS")]
        wallet_address: String,
    },
    /// Register a new node to an existing user, or link an existing node to a user.
    RegisterNode {
        /// ID of the node to register. If not provided, a new node will be created.
        #[arg(long, value_name = "NODE_ID")]
        node_id: Option<u64>,
    },
    /// Clear the node configuration and logout.
    Logout,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let nexus_environment_str = std::env::var("NEXUS_ENVIRONMENT").unwrap_or_default();
    let environment = nexus_environment_str
        .parse::<Environment>()
        .unwrap_or(Environment::default());

    let config_path = get_config_path()?;
    let args = Args::parse();
    match args.command {
        Command::Start {
            node_id,
            max_threads,
        } => {
            let mut node_id = node_id;
            // If no node ID is provided, try to load it from the config file.
            if node_id.is_none() && config_path.exists() {
                if let Ok(config) = Config::load_from_file(&config_path) {
                    if let Ok(id) = config.node_id.parse::<u64>() {
                        node_id = Some(id);
                    }
                }
            }
            start(node_id, environment, max_threads)
        }
        Command::Logout => {
            println!("Logging out and clearing node configuration file...");
            Config::clear_node_config(&config_path).map_err(Into::into)
        }
        Command::RegisterUser { wallet_address } => {
            println!(
                "Registering user with wallet address: {} in environment: {:?}",
                wallet_address, environment
            );
            // Check if the wallet address is valid
            if !keys::is_valid_eth_address(&wallet_address) {
                let err_msg = format!(
                    "Invalid Ethereum wallet address: {}. It should be a 42-character hex string starting with '0x'.",
                    wallet_address
                );
                return Err(Box::from(err_msg));
            }
            let orchestrator_client = OrchestratorClient::new(environment);
            let uuid = uuid::Uuid::new_v4().to_string();
            match orchestrator_client
                .register_user(&uuid, &wallet_address)
                .await
            {
                Ok(_) => println!("User {} registered successfully.", uuid),
                Err(e) => {
                    eprintln!("Failed to register user: {}", e);
                    return Err(e);
                }
            }

            // Save the configuration file with the user ID and wallet address
            let config = Config::new(
                uuid,
                wallet_address,
                String::new(), // node_id is empty for now
                environment,
            );
            config
                .save(&config_path)
                .map_err(|e| format!("Failed to save config: {}", e))?;
            Ok(())
        }
        Command::RegisterNode { node_id } => {
            // Register a new node, or link an existing node to a user.
            // Requires: a config file with a registered user
            // If a node_id is provided, update the config with it and use it.
            // If no node_id is provided, generate a new one.
            let mut config = Config::load_from_file(&config_path).map_err(|e| {
                format!("Failed to load config: {}. Please register a user first", e)
            })?;
            if config.user_id.is_empty() {
                return Err(Box::from(
                    "No user registered. Please register a user first.",
                ));
            }
            if let Some(node_id) = node_id {
                // If a node_id is provided, update the config with it.
                println!("Registering node ID: {}", node_id);
                config.node_id = node_id.to_string();
                config
                    .save(&config_path)
                    .map_err(|e| format!("Failed to save updated config: {}", e))?;
                println!("Successfully registered node with ID: {}", node_id);
                Ok(())
            } else {
                println!(
                    "No node ID provided. Registering a new node in environment: {:?}",
                    environment
                );
                let orchestrator_client = OrchestratorClient::new(environment);
                match orchestrator_client.register_node(&config.user_id).await {
                    Ok(node_id) => {
                        println!("Node registered successfully with ID: {}", node_id);
                        // Update the config with the new node ID
                        let mut updated_config = config;
                        updated_config.node_id = node_id;
                        updated_config
                            .save(&config_path)
                            .map_err(|e| format!("Failed to save updated config: {}", e))?;
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("Failed to register node: {}", e);
                        Err(e)
                    }
                }
            }
        }
    }
}

/// Starts the Nexus CLI application.
///
/// # Arguments
/// * `node_id` - This client's unique identifier, if available.
/// * `env` - The environment to connect to.
/// * `max_threads` - Optional maximum number of threads to use for proving.
fn start(
    node_id: Option<u64>,
    env: Environment,
    _max_threads: Option<u32>,
) -> Result<(), Box<dyn Error>> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Initialize the terminal with Crossterm backend.
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create the application and run it.
    let orchestrator_client = OrchestratorClient::new(env);
    // TODO: Persist the signing key in the config file.
    let mut csprng = rand_core::OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let app = ui::App::new(node_id, orchestrator_client, signing_key);
    let res = ui::run(&mut terminal, app);

    // Clean up the terminal after running the application.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res?;
    Ok(())
}
