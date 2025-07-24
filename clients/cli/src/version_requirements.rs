use reqwest::Client;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

const CONFIG_URL: &str = "https://cli.nexus.xyz/version.json";
#[cfg(test)]
const TEST_ERROR_URL: &str = "https://cli.nexus.xyz/nonexistent.json";
const CONFIG_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Error, Debug)]
pub enum VersionRequirementsError {
    #[error("Failed to fetch config: {0}")]
    Fetch(String),

    #[error("Failed to parse config JSON: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("Failed to parse version: {0}")]
    VersionParse(#[from] semver::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionRequirements {
    pub version_constraints: Vec<VersionConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionConstraint {
    pub version: String,
    #[serde(rename = "type")]
    pub constraint_type: ConstraintType,
    pub message: String,
    #[serde(default)]
    pub start_date: Option<u64>, // Unix timestamp, optional
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConstraintType {
    Blocking,
    Warning,
    Notice,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VersionCheckResult {
    pub constraint_type: ConstraintType,
    pub message: String,
}

impl VersionRequirements {
    /// Fetch version requirements from the remote config file
    pub async fn fetch() -> Result<Self, VersionRequirementsError> {
        let client = Client::builder()
            .timeout(CONFIG_TIMEOUT)
            .user_agent("nexus-cli/version-checker")
            .build()
            .expect("Failed to create HTTP client");

        let response = client
            .get(CONFIG_URL)
            .send()
            .await
            .map_err(|e| VersionRequirementsError::Fetch(e.to_string()))?;

        if !response.status().is_success() {
            let error_msg = format!("HTTP {}: {}", response.status(), response.status().as_str());
            return Err(VersionRequirementsError::Fetch(error_msg));
        }

        // Get the response body as text first for debugging
        let response_text = response.text().await.map_err(|e| {
            VersionRequirementsError::Fetch(format!("Failed to read response body: {}", e))
        })?;

        // Try to parse the JSON
        let config: VersionRequirements =
            serde_json::from_str(&response_text).map_err(VersionRequirementsError::Parse)?;
        Ok(config)
    }

    /// Check all version constraints and return the most severe violation
    pub fn check_version_constraints(
        &self,
        current_version: &str,
        latest_version: Option<&str>,
        release_url: Option<&str>,
    ) -> Result<Option<VersionCheckResult>, VersionRequirementsError> {
        let current = Version::parse(current_version.strip_prefix('v').unwrap_or(current_version))?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut most_severe_violation: Option<VersionCheckResult> = None;

        for constraint in &self.version_constraints {
            // Check if constraint is active (no start date or start date has passed)
            if let Some(start_date) = constraint.start_date {
                if now < start_date {
                    continue; // Constraint not yet active
                }
            }

            let min_version = Version::parse(&constraint.version)?;

            if current < min_version {
                // This constraint is violated
                let message = self.format_message(
                    &constraint.message,
                    current_version,
                    &constraint.version,
                    latest_version,
                    release_url,
                );

                let result = VersionCheckResult {
                    constraint_type: constraint.constraint_type.clone(),
                    message,
                };

                // Determine if this is more severe than the current most severe
                let should_replace = match (&most_severe_violation, &constraint.constraint_type) {
                    (None, _) => true, // First violation found
                    (Some(_existing), ConstraintType::Blocking) => {
                        // Blocking always takes precedence
                        true
                    }
                    (Some(existing), ConstraintType::Warning) => {
                        // Warning takes precedence over Notice
                        matches!(existing.constraint_type, ConstraintType::Notice)
                    }
                    (Some(_existing), ConstraintType::Notice) => {
                        // Notice only takes precedence if existing is also Notice
                        matches!(_existing.constraint_type, ConstraintType::Notice)
                    }
                };

                if should_replace {
                    most_severe_violation = Some(result);
                }
            }
        }

        Ok(most_severe_violation)
    }

    /// Format a message template with the given variables
    fn format_message(
        &self,
        template: &str,
        current_version: &str,
        version: &str,
        latest_version: Option<&str>,
        release_url: Option<&str>,
    ) -> String {
        template
            .replace("{current}", current_version)
            .replace("{version}", version)
            .replace("{latest}", latest_version.unwrap_or("unknown"))
            .replace(
                "{release_url}",
                release_url.unwrap_or("https://github.com/nexus-xyz/nexus-cli/releases"),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        let config = VersionRequirements {
            version_constraints: vec![
                VersionConstraint {
                    version: "0.9.0".to_string(),
                    constraint_type: ConstraintType::Warning,
                    message: "Warning: {current} < {version}".to_string(),
                    start_date: None,
                },
                VersionConstraint {
                    version: "0.8.0".to_string(),
                    constraint_type: ConstraintType::Blocking,
                    message: "Blocking: {current} < {version}".to_string(),
                    start_date: None,
                },
            ],
        };

        // Test constraint checking
        let result = config
            .check_version_constraints("0.9.1", None, None)
            .unwrap();
        assert!(result.is_none()); // No violations

        let result = config
            .check_version_constraints("0.8.9", None, None)
            .unwrap();
        assert!(result.is_some());
        assert!(matches!(
            result.unwrap().constraint_type,
            ConstraintType::Warning
        ));

        let result = config
            .check_version_constraints("0.7.9", None, None)
            .unwrap();
        assert!(result.is_some());
        assert!(matches!(
            result.unwrap().constraint_type,
            ConstraintType::Blocking
        ));
    }

    #[test]
    fn test_version_parsing() {
        let config = VersionRequirements {
            version_constraints: vec![
                VersionConstraint {
                    version: "1.0.0".to_string(),
                    constraint_type: ConstraintType::Warning,
                    message: "Warning: {current} < {version}".to_string(),
                    start_date: None,
                },
                VersionConstraint {
                    version: "0.1.0".to_string(),
                    constraint_type: ConstraintType::Blocking,
                    message: "Blocking: {current} < {version}".to_string(),
                    start_date: None,
                },
            ],
        };

        // Test that versions with 'v' prefix are handled correctly
        let result = config
            .check_version_constraints("v1.0.0", None, None)
            .unwrap();
        assert!(result.is_none()); // No violations
    }

    #[test]
    fn test_constraint_priority() {
        let config = VersionRequirements {
            version_constraints: vec![
                VersionConstraint {
                    version: "0.9.0".to_string(),
                    constraint_type: ConstraintType::Notice,
                    message: "Notice: {current} < {version}".to_string(),
                    start_date: None,
                },
                VersionConstraint {
                    version: "0.8.0".to_string(),
                    constraint_type: ConstraintType::Warning,
                    message: "Warning: {current} < {version}".to_string(),
                    start_date: None,
                },
                VersionConstraint {
                    version: "0.7.0".to_string(),
                    constraint_type: ConstraintType::Blocking,
                    message: "Blocking: {current} < {version}".to_string(),
                    start_date: None,
                },
            ],
        };

        // Test that blocking takes precedence over warning and notice
        let result = config
            .check_version_constraints("0.6.0", None, None)
            .unwrap();
        assert!(result.is_some());
        assert!(matches!(
            result.unwrap().constraint_type,
            ConstraintType::Blocking
        ));
    }

    #[test]
    fn test_message_formatting() {
        let config = VersionRequirements {
            version_constraints: vec![VersionConstraint {
                version: "1.0.0".to_string(),
                constraint_type: ConstraintType::Notice,
                message: "Version {current} < {version}. Latest: {latest}. URL: {release_url}"
                    .to_string(),
                start_date: None,
            }],
        };

        let result = config
            .check_version_constraints("0.9.0", Some("1.1.0"), Some("https://example.com"))
            .unwrap();
        assert!(result.is_some());
        let message = &result.unwrap().message;
        assert!(message.contains("0.9.0"));
        assert!(message.contains("1.0.0"));
        assert!(message.contains("1.1.0"));
        assert!(message.contains("https://example.com"));
    }
}
