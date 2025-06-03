// Copyright (c) 2024 Nexus. All rights reserved.

mod analytics;
mod config;
mod environment;
#[path = "proto/nexus.orchestrator.rs"]
mod nexus_orchestrator;
mod orchestrator_client;
mod prover;
mod ui;
mod utils;
use crate::config::Config;
use crate::environment::Environment;
use crate::orchestrator_client::OrchestratorClient;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::path::PathBuf;
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
    /// Logout from the current session
    Logout,
}

/// Get the path to the Nexus config file, typically located at ~/.nexus/config.json.
fn get_config_path() -> Result<PathBuf, ()> {
    let home_path = home::home_dir().expect("Failed to get home directory");
    let config_path = home_path.join(".nexus").join("config.json");
    Ok(config_path)
}

fn main() -> Result<(), Box<dyn Error>> {
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
                    let node_id_as_u64 = config
                        .node_id
                        .parse::<u64>()
                        .expect("Failed to parse node ID");
                    node_id = Some(node_id_as_u64);
                }
            }

            let environment = env.unwrap_or_default();
            start(node_id, environment, max_threads)
        }
        Command::Logout => {
            let config_path = get_config_path().expect("Failed to get config path");
            Config::clear_node_config(&config_path).map_err(Into::into)
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
