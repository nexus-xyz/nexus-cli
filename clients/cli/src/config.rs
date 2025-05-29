//! Application configuration.

use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub node_id: String,
}

impl Config {
    /// Create Config with the given node_id.
    #[allow(unused)]
    pub fn new(node_id: String) -> Self {
        Config { node_id }
    }

    /// Loads configuration from a JSON file at the given path.
    ///
    /// # Errors
    /// Returns an `std::io::Error` if reading from file fails or JSON is invalid.
    pub fn load_from_file(path: &Path) -> Result<Self, std::io::Error> {
        let buf = fs::read(path)?;
        let config: Config = serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    /// Saves the configuration to a JSON file at the given path.
    ///
    /// Directories will be created if they don't exist. This method overwrites existing files.
    ///
    /// # Errors
    /// Returns an `std::io::Error` if writing to file fails or serialization fails.
    #[allow(unused)]
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    // Loading a saved configuration file should return the same configuration.
    fn test_load_recovers_saved_config() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");

        let config = Config::new("test_node_id".to_string());
        config.save(&path).unwrap();

        let loaded_config = Config::load_from_file(&path).unwrap();
        assert_eq!(config, loaded_config);
    }

    #[test]
    // Saving a configuration should create directories if they don't exist.
    fn test_save_creates_directories() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent_dir").join("config.json");

        // Attempt to save the configuration
        let config = Config::new("test_node_id".to_string());
        let result = config.save(&path);

        // Check if the directories were created
        assert!(result.is_ok(), "Failed to save config");
        assert!(
            path.parent().unwrap().exists(),
            "Parent directory does not exist"
        );
    }

    #[test]
    // Saving a configuration should overwrite an existing file.
    fn test_save_overwrites_existing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");

        // Create an initial config and save it
        let config1 = Config::new("test_node_id_1".to_string());
        config1.save(&path).unwrap();

        // Create a new config and save it to the same path
        let config2 = Config::new("test_node_id_2".to_string());
        config2.save(&path).unwrap();

        // Load the saved config and check if it matches the second one
        let loaded_config = Config::load_from_file(&path).unwrap();
        assert_eq!(config2, loaded_config);
    }

    #[test]
    // Loading an invalid JSON file should return an error.
    fn test_load_rejects_invalid_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("invalid_config.json");

        let mut file = File::create(&path).unwrap();
        writeln!(file, "invalid json").unwrap();

        let result = Config::load_from_file(&path);
        assert!(result.is_err());
    }
}
