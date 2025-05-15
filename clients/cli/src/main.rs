// Copyright (c) 2024 Nexus. All rights reserved.

mod analytics;
mod config;
mod flops;
mod memory_stats;
#[path = "proto/nexus.orchestrator.rs"]
mod nexus_orchestrator;
mod node_id_manager;
mod orchestrator_client;
mod prover;
mod setup;
mod utils;

// Update the import path to use the proto module
use clap::{Parser, Subcommand};
use log::error;
use crate::orchestrator_client::OrchestratorClient;
use crate::prover::start_prover;
use crate::setup::SetupResult;

#[derive(clap::ValueEnum, Clone, Debug)]
enum Environment {
    Local,
    Dev,
    Staging,
    Beta,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Command to execute
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the prover
    Start {
        /// Environment to connect to.
        #[arg(long, value_enum)]
        env: Option<Environment>,

        /// Number of threads to use for proving.
        #[arg(long, default_value_t = 1)]
        num_threads: usize,
    },
    /// Logout from the current session
    Logout,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Command::Start { env, num_threads } => {
            utils::cli_branding::print_banner();

            let orchestrator_client = {
                let environment = config::Environment::from_args(env.as_ref());
                OrchestratorClient::new(environment)
            };
            
            // Run initial setup
            match setup::run_initial_setup().await {
                SetupResult::Anonymous => {
                    println!("Proving anonymously...");
                    start_prover(orchestrator_client, None, num_threads).await?;
                }
                SetupResult::Connected(node_id) => {
                    println!("Proving with existing node id: {}", node_id);
                    let node_id: u64 = node_id.parse().expect(format!("invalid node id {}", node_id).as_str());
                    start_prover(orchestrator_client, Some(node_id), num_threads).await?;
                }
                SetupResult::Invalid => {
                    error!("Invalid setup option selected.");
                    return Err("Invalid setup option selected".into());
                }
            }
        }
        Command::Logout => match setup::clear_node_id() {
            Ok(_) => println!("Successfully logged out"),
            Err(e) => eprintln!("Failed to logout: {}", e),
        },
    }

    Ok(())
}
