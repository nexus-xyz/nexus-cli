// Copyright (c) 2024 Nexus. All rights reserved.

mod analytics;
mod config;
mod consts;
mod environment;
mod error_classifier;
mod events;
mod keys;
mod logging;
#[path = "proto/nexus.orchestrator.rs"]
mod nexus_orchestrator;
mod orchestrator;
mod pretty;
mod prover;
mod prover_runtime;
mod register;
pub mod system;
mod task;
mod task_cache;
mod ui;
mod version_checker;
mod version_requirements;
mod workers;

use crate::config::{Config, get_config_path};
use crate::environment::Environment;
use crate::orchestrator::{Orchestrator, OrchestratorClient};
use crate::pretty::print_cmd_info;
use crate::prover_runtime::{start_anonymous_workers, start_authenticated_workers};
use crate::register::{register_node, register_user};
use crate::version_requirements::{VersionRequirements, VersionRequirementsError};
use clap::{ArgAction, Parser, Subcommand};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ed25519_dalek::SigningKey;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{error::Error, io};
use tokio::sync::broadcast;

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

        /// Run without the terminal UI
        #[arg(long = "headless", action = ArgAction::SetTrue)]
        headless: bool,

        /// Maximum number of threads to use for proving.
        #[arg(long = "max-threads", value_name = "MAX_THREADS")]
        max_threads: Option<u32>,

        /// Custom orchestrator URL (overrides environment setting)
        #[arg(long = "orchestrator-url", value_name = "URL")]
        orchestrator_url: Option<String>,

        /// Disable background colors in the dashboard
        #[arg(long = "no-background-color", action = ArgAction::SetTrue)]
        no_background_color: bool,
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
            headless,
            max_threads,
            orchestrator_url,
            no_background_color,
        } => {
            // If a custom orchestrator URL is provided, create a custom environment
            let final_environment = if let Some(url) = orchestrator_url {
                Environment::Custom {
                    orchestrator_url: url,
                }
            } else {
                environment
            };
            start(
                node_id,
                final_environment,
                config_path,
                headless,
                max_threads,
                no_background_color,
            )
            .await
        }
        Command::Logout => {
            print_cmd_info!("Logging out", "Clearing node configuration file...");
            Config::clear_node_config(&config_path).map_err(Into::into)
        }
        Command::RegisterUser { wallet_address } => {
            print_cmd_info!("Registering user", "Wallet address: {}", wallet_address);
            let orchestrator = Box::new(OrchestratorClient::new(environment));
            register_user(&wallet_address, &config_path, orchestrator).await
        }
        Command::RegisterNode { node_id } => {
            let orchestrator = Box::new(OrchestratorClient::new(environment));
            register_node(node_id, &config_path, orchestrator).await
        }
    }
}

/// Starts the Nexus CLI application.
///
/// # Arguments
/// * `node_id` - This client's unique identifier, if available.
/// * `env` - The environment to connect to.
/// * `config_path` - Path to the configuration file.
/// * `headless` - If true, runs without the terminal UI.
/// * `max_threads` - Optional maximum number of threads to use for proving.
async fn start(
    node_id: Option<u64>,
    env: Environment,
    config_path: std::path::PathBuf,
    headless: bool,
    max_threads: Option<u32>,
    no_background_color: bool,
) -> Result<(), Box<dyn Error>> {
    // Check version requirements before starting any workers
    match VersionRequirements::fetch().await {
        Ok(requirements) => {
            let current_version = env!("CARGO_PKG_VERSION");
            match requirements.check_version_constraints(current_version, None, None) {
                Ok(Some(violation)) => match violation.constraint_type {
                    crate::version_requirements::ConstraintType::Blocking => {
                        eprintln!("❌ Version requirement not met: {}", violation.message);
                        std::process::exit(1);
                    }
                    crate::version_requirements::ConstraintType::Warning => {
                        eprintln!("⚠️  {}", violation.message);
                    }
                    crate::version_requirements::ConstraintType::Notice => {
                        eprintln!("ℹ️  {}", violation.message);
                    }
                },
                Ok(None) => {
                    // No violations found, continue
                }
                Err(e) => {
                    eprintln!("❌ Failed to parse version requirements: {}", e);
                    eprintln!(
                        "If this issue persists, please file a bug report at: https://github.com/nexus-xyz/nexus-cli/issues/new"
                    );
                    std::process::exit(1);
                }
            }
        }
        Err(VersionRequirementsError::Fetch(e)) => {
            eprintln!("❌ Failed to fetch version requirements: {}", e);
            eprintln!(
                "If this issue persists, please file a bug report at: https://github.com/nexus-xyz/nexus-cli/issues/new"
            );
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("❌ Failed to check version requirements: {}", e);
            eprintln!(
                "If this issue persists, please file a bug report at: https://github.com/nexus-xyz/nexus-cli/issues/new"
            );
            std::process::exit(1);
        }
    }

    let mut node_id = node_id;

    // If no node ID is provided, try to load it from the config file.
    if node_id.is_none() && config_path.exists() {
        let config = Config::load_from_file(&config_path)?;

        // Check if user is registered but node_id is missing or invalid
        if !config.user_id.is_empty() {
            if config.node_id.is_empty() {
                print_cmd_info!(
                    "✅ User registered, but no node found.",
                    "Please register a node to continue: nexus-cli register-node"
                );
                return Err(
                    "Node registration required. Please run 'nexus-cli register-node' first."
                        .into(),
                );
            }

            match config.node_id.parse::<u64>() {
                Ok(id) => {
                    node_id = Some(id);
                    print_cmd_info!("✅ Found Node ID from config file", "Node ID: {}", id);
                }
                Err(_) => {
                    print_cmd_info!(
                        "❌ Invalid node ID in config file.",
                        "Please register a new node: nexus-cli register-node"
                    );
                    return Err("Invalid node ID in config. Please run 'nexus-cli register-node' to fix this.".into());
                }
            }
        } else {
            print_cmd_info!(
                "❌ No user registration found.",
                "Please register your wallet address first: nexus-cli register-user --wallet-address <your-wallet-address>"
            );
            return Err("User registration required. Please run 'nexus-cli register-user --wallet-address <your-wallet-address>' first.".into());
        }
    } else if node_id.is_none() {
        // No config file exists at all
        print_cmd_info!(
            "Welcome to Nexus CLI!",
            "Please register your wallet address to get started: nexus-cli register-user --wallet-address <your-wallet-address>"
        );
    }

    // Create a signing key for the prover.
    let mut csprng = rand_core::OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let orchestrator_client = OrchestratorClient::new(env.clone());
    // Clamp the number of workers to [1,8]. Keep this low for now to avoid rate limiting.
    let num_workers: usize = max_threads.unwrap_or(1).clamp(1, 8) as usize;
    let (shutdown_sender, _) = broadcast::channel(1); // Only one shutdown signal needed

    // Get client_id for analytics - use wallet address from API if available, otherwise "anonymous"
    let client_id = if let Some(node_id) = node_id {
        match orchestrator_client.get_node(&node_id.to_string()).await {
            Ok(wallet_address) => {
                // Use wallet address as client_id for analytics
                wallet_address
            }
            Err(_) => {
                // If API call fails, use "anonymous" regardless of config
                "anonymous".to_string()
            }
        }
    } else {
        // No node_id available, use "anonymous"
        "anonymous".to_string()
    };

    let (mut event_receiver, mut join_handles) = match node_id {
        Some(node_id) => {
            start_authenticated_workers(
                node_id,
                signing_key.clone(),
                orchestrator_client.clone(),
                num_workers,
                shutdown_sender.subscribe(),
                env.clone(),
                client_id,
            )
            .await
        }
        None => {
            start_anonymous_workers(num_workers, shutdown_sender.subscribe(), env, client_id).await
        }
    };

    if !headless {
        // Terminal setup
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        // Initialize the terminal with Crossterm backend.
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Create the application and run it.
        let app = ui::App::new(
            node_id,
            orchestrator_client.environment().clone(),
            event_receiver,
            shutdown_sender,
            no_background_color,
        );
        let res = ui::run(&mut terminal, app).await;

        // Clean up the terminal after running the application.
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        res?;
    } else {
        // Headless mode: log events to console.

        // Trigger shutdown on Ctrl+C
        let shutdown_sender_clone = shutdown_sender.clone();
        tokio::spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                let _ = shutdown_sender_clone.send(());
            }
        });

        let mut shutdown_receiver = shutdown_sender.subscribe();
        loop {
            tokio::select! {
                Some(event) = event_receiver.recv() => {
                    println!("{}", event);
                }
                _ = shutdown_receiver.recv() => {
                    break;
                }
            }
        }
    }
    println!("\nExiting...");
    for handle in join_handles.drain(..) {
        let _ = handle.await;
    }
    println!("Nexus CLI application exited successfully.");
    Ok(())
}
