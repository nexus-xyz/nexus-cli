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

use crate::prover::start_prover;
use crate::setup::SetupResult;
use clap::{Parser, Subcommand};
use colored::Colorize;
use log::error;
use std::error::Error;

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
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize default log level, but can be overridden by the RUST_LOG environment variable.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();

    match cli.command {
        Command::Start { env, num_threads } => {
            utils::cli_branding::print_banner();
            println!();
            println!(
                "{}: {}",
                "Computational capacity of this node".bold(),
                format!("{:.2} GFLOPS", flops::measure_gflops()).bright_cyan()
            );

            let environment = config::Environment::from_args(env.as_ref());
            println!(
                "{}: {}",
                "Environment".bold(),
                environment.to_string().bright_cyan()
            );

            // Run initial setup
            match setup::run_initial_setup().await {
                SetupResult::Anonymous => {
                    println!("Proving anonymously...");
                    start_prover(environment, None, num_threads).await?;
                }
                SetupResult::Connected(node_id) => {
                    println!("Proving with existing node id: {}", node_id);
                    let node_id: u64 = node_id
                        .parse()
                        .unwrap_or_else(|_| panic!("invalid node id {}", node_id));
                    start_prover(environment, Some(node_id), num_threads).await?;
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
