use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use std::path::PathBuf;

/// Helper to get a temporary config directory
fn temp_config_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("create temp dir")
}

/// Helper to get config file path in the temp dir
fn config_file_path(dir: &tempfile::TempDir) -> PathBuf {
    dir.path().join(".nexus").join("config.json")
}

const BINARY_NAME: &str = "nexus-network";

#[test]
/// Help command should display usage information.
fn cli_help_displays_usage() {
    let mut cmd = Command::cargo_bin(BINARY_NAME).unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("Command-line arguments"));
}

#[test]
#[ignore] // This currently involves network calls and creating a config file.
fn register_user_command_creates_config_file() {
    let tmp = temp_config_dir();
    let config_path = config_file_path(&tmp);
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();

    // Ensure the file does not exist initially
    assert!(!config_path.exists());

    // Run the command
    let mut cmd = Command::cargo_bin(BINARY_NAME).unwrap();
    cmd.arg("register-user")
        .arg("--wallet-address")
        .arg("0x1234567890abcdef1234567890abcdef12345600")
        .env("HOME", tmp.path()) // simulate different $HOME
        .assert()
        .success()
        .stdout(contains("User registered successfully"));

    // Confirm the file was created
    assert!(config_path.exists());
}

#[test]
/// Logout command should delete an existing config file.
fn logout_deletes_config_file() {
    let tmp = temp_config_dir();
    let config_path = config_file_path(&tmp);
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    fs::write(&config_path, "{}").unwrap();

    // Ensure the file exists
    assert!(config_path.exists());

    // Run the command
    let mut cmd = Command::cargo_bin(BINARY_NAME).unwrap();
    cmd.arg("logout")
        .env("HOME", tmp.path()) // simulate different $HOME
        .assert()
        .success()
        .stdout(contains("Logging out"));

    // Confirm the file was deleted
    assert!(!config_path.exists());
}
