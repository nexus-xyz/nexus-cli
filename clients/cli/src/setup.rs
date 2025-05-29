use crate::config::Config;
use colored::Colorize;
use std::fs;
use std::io::stdin;
use std::path::Path;

#[allow(unused)]
pub enum SetupResult {
    /// The user is in anonymous mode
    Anonymous,
    /// Connected with the given node id
    Connected(String),
    /// Failed to connect.
    Invalid,
}

/// Run the initial setup for the Nexus CLI.
///
/// Checks for, and reads or creates the config file at the given path.
#[allow(unused)]
pub async fn run_initial_setup(config_path: &Path) -> Result<SetupResult, std::io::Error> {
    if config_path.exists() {
        // If a config file exists, attempt to read the node ID from it.
        let node_config = Config::load_from_file(config_path)?;
        let node_id = node_config.node_id;
        println!(
            "\nThis node is already connected to an account using node id: {}",
            node_id
        );
        if std::env::var_os("NONINTERACTIVE").is_some() {
            return Ok(SetupResult::Connected(node_id));
        }

        println!("Do you want to use the existing user account? [Y/n]");
        let use_existing_config = {
            let mut buf = String::new();
            stdin().read_line(&mut buf)?;
            !buf.trim().eq_ignore_ascii_case("n")
        };

        if use_existing_config {
            return Ok(SetupResult::Connected(node_id));
        } else {
            println!("Ignoring existing node id...");
        }
    } else {
        println!("\nThis node is not connected to any account.\n");
    }

    println!("[1] Enter '1' Anonymous mode: start proving without earning Devnet points");
    println!("[2] Enter '2' Authenticated mode: start proving and earning Devnet points");

    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    let option = buf.trim();

    match option {
        "1" => {
            println!("You chose option 1\n");
            Ok(SetupResult::Anonymous)
        }
        "2" => {
            println!(
                "\n===== {} =====\n",
                "Adding your node ID to the CLI"
                    .bold()
                    .underline()
                    .bright_cyan()
            );
            println!("You chose to start earning Devnet points by connecting your node ID\n");
            println!("If you don't have a node ID, you can get it by following these steps:\n");
            println!("1. Go to https://app.nexus.xyz/nodes");
            println!("2. Sign in");
            println!("3. Click on the '+ Add Node' button");
            println!("4. Select 'Add CLI node'");
            println!("5. You will be given a node ID to add to this CLI");
            println!("6. Enter the node ID into the terminal below:\n");

            let node_id = get_node_id_from_user();
            let node_config = Config::new(node_id.clone());
            node_config.save(config_path)?;
            Ok(SetupResult::Connected(node_id))
        }
        _ => {
            println!("Invalid option {}", option);
            Ok(SetupResult::Invalid)
        }
    }
}

/// Get the node ID from the user input.
fn get_node_id_from_user() -> String {
    println!("{}", "Please enter your node ID:".green());
    let mut node_id = String::new();
    std::io::stdin()
        .read_line(&mut node_id)
        .expect("Failed to read node ID");
    node_id.trim().to_string()
}

/// Clear the node ID configuration file.
pub fn clear_node_config(path: &Path) -> std::io::Result<()> {
    // Check that the path ends with config.json
    if !path.ends_with("config.json") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Path must end with config.json",
        ));
    }

    // If no file exists, return OK
    if !path.exists() {
        println!("No config file found at {}", path.display());
        return Ok(());
    }

    // If the file exists, remove it
    fs::remove_file(path)
}
