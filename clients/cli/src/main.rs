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

use crate::config::{get_config_path, Config};
use crate::environment::Environment;
use crate::orchestrator_client::OrchestratorClient;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
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

        /// Environment to connect to.
        #[arg(long, value_enum)]
        env: Option<Environment>,

        /// Maximum number of threads to use for proving.
        #[arg(long)]
        max_threads: Option<u32>,
    },
    /// Register a new user
    RegisterUser {
        /// Environment to connect to.
        #[arg(long, value_enum)]
        env: Option<Environment>,

        /// User's public Ethereum wallet address. 42-character hex string starting with '0x'
        wallet_address: String,
    },
    /// Register a new node to an existing user
    RegisterNode {
        /// Environment to connect to.
        #[arg(long, value_enum)]
        env: Option<Environment>,
    },
    /// Clear the node configuration and logout.
    Logout,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    match args.command {
        Command::Start {
            node_id,
            env,
            max_threads,
        } => {
            let mut node_id = node_id;
            // If no node ID is provided, try to load it from the config file.
            let config_path = get_config_path().expect("Failed to get config path");
            if node_id.is_none() && config_path.exists() {
                if let Ok(config) = Config::load_from_file(&config_path) {
                    if let Ok(node_id_as_u64) = config.node_id.parse::<u64>() {
                        node_id = Some(node_id_as_u64);
                    }
                }
            }
            let environment = env.unwrap_or_default();
            start(node_id, environment, max_threads)
        }
        Command::Logout => {
            println!("Logging out and clearing node configuration file...");
            let config_path = get_config_path().expect("Failed to get config path");
            Config::clear_node_config(&config_path).map_err(Into::into)
        }
        Command::RegisterUser {
            env,
            wallet_address,
        } => {
            let environment = env.unwrap_or_default();
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
                    return Err(e.into());
                }
            }
            // TODO: save the user ID to the config file
            let config = Config::new(uuid, String::new());
            let config_path = get_config_path().expect("Failed to get config path");
            config
                .save(&config_path)
                .map_err(|e| format!("Failed to save config: {}", e))?;
            Ok(())
        }
        Command::RegisterNode { env } => {
            let environment = env.unwrap_or_default();
            println!("Registering node in environment: {:?}", environment);
            // Check if the user is registered
            let config_path = get_config_path().expect("Failed to get config path");
            if !config_path.exists() {
                return Err(Box::from(
                    "No user registered. Please register a user first.",
                ));
            }
            let config = Config::load_from_file(&config_path)
                .map_err(|e| format!("Failed to load config: {}", e))?;
            if config.user_id.is_empty() {
                return Err(Box::from(
                    "No user registered. Please register a user first.",
                ));
            }
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
                    Err(e.into())
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
    let app = ui::App::new(node_id, env, orchestrator_client);
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
