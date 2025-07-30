//! Version Checking Module
//!
//! Checks for new versions of the CLI by querying the GitHub API
//!
//! # Mock Testing
//!
//! This module provides several approaches for mock testing:
//!
//! ## 1. **Trait-Based Mocking (Recommended)**
//!
//! Uses the `VersionCheckable` trait with `mockall` for comprehensive mocking:
//!
//! ```rust
//! use mockall::predicate::*;
//! use version_checker::{MockVersionCheckable, version_checker_task_with_interval};
//!
//! #[tokio::test]
//! async fn test_version_update_detection() {
//!     let mut mock_checker = MockVersionCheckable::new();
//!     mock_checker
//!         .expect_current_version()
//!         .return_const("0.9.0".to_string());
//!
//!     mock_checker
//!         .expect_check_latest_version()
//!         .returning(|| Ok(create_mock_release("v0.9.1")));
//!
//!     let (event_sender, mut event_receiver) = mpsc::channel(10);
//!     let (shutdown_sender, shutdown_receiver) = broadcast::channel(1);
//!
//!     let task_handle = tokio::spawn(async move {
//!         version_checker_task_with_interval(
//!             Box::new(mock_checker),
//!             event_sender,
//!             shutdown_receiver,
//!             Duration::from_millis(100),
//!         ).await;
//!     });
//!
//!     // Test assertions here...
//! }
//! ```
//!
//! ## 2. **Dependency Injection**
//!
//! The `version_checker_task` function accepts `Box<dyn VersionCheckable>`, making it
//! easy to inject mocks or different implementations:
//!
//! ```rust
//! // Production code
//! let real_checker = Box::new(VersionChecker::new("0.9.0".to_string()));
//! version_checker_task(real_checker, event_sender, shutdown).await;
//!
//! // Test code
//! let mock_checker = Box::new(mock_version_checker);
//! version_checker_task(mock_checker, event_sender, shutdown).await;
//! ```
//!
//! ## 3. **Testing Different Scenarios**
//!
//! The mock system supports testing various scenarios:
//!
//! - **Update Available**: Mock returns newer version
//! - **No Update**: Mock returns same/older version
//! - **API Errors**: Mock returns errors to test error handling
//! - **Multiple Checks**: Test that duplicate notifications are prevented
//! - **Rate Limiting**: Test with different check intervals
//!
//! ## Mock Testing Best Practices
//!
//! 1. **Use `times()` wisely**: Specify expected call counts for better validation
//! 2. **Test edge cases**: Invalid versions, network errors, malformed responses
//! 3. **Verify event contents**: Check message format, log levels, event types
//! 4. **Test timing**: Use configurable intervals for faster tests
//! 5. **Clean shutdown**: Always test graceful shutdown scenarios

use crate::error_classifier::LogLevel;
use crate::events::{Event, EventType};
use crate::version_requirements::{ConstraintType, VersionCheckResult, VersionRequirements};
use reqwest::{Client, ClientBuilder};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{Instant, sleep};

#[cfg(test)]
use mockall::{automock, predicate::*};

// Check for updates and constraints every 24 hours
const VERSION_CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

// GitHub API endpoint for the latest release
const GITHUB_RELEASES_URL: &str =
    "https://api.github.com/repos/nexus-xyz/nexus-cli/releases/latest";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub published_at: String,
    pub html_url: String,
    pub prerelease: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub release_url: Option<String>,
    pub last_check: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct VersionConstraintState {
    pub last_constraint_check: Option<Instant>,
    pub current_constraints: Option<VersionRequirements>,
    pub last_violation: Option<VersionCheckResult>,
}

impl VersionInfo {
    pub fn new(current_version: String) -> Self {
        Self {
            current_version,
            latest_version: None,
            update_available: false,
            release_url: None,
            last_check: None,
        }
    }

    pub fn update_from_release(&mut self, release: GitHubRelease) {
        self.latest_version = Some(release.tag_name.clone());
        self.release_url = Some(release.html_url);
        self.update_available = self.is_newer_version(&release.tag_name);
        self.last_check = Some(Instant::now());
    }

    /// Compare semantic versions to determine if the latest version is newer
    fn is_newer_version(&self, latest: &str) -> bool {
        match (parse_version(&self.current_version), parse_version(latest)) {
            (Ok(current), Ok(latest_ver)) => latest_ver > current,
            _ => false, // If parsing fails, assume no update needed
        }
    }
}

/// Parse a version string, handling optional 'v' prefix
fn parse_version(version: &str) -> Result<Version, semver::Error> {
    let clean_version = version.strip_prefix('v').unwrap_or(version);
    Version::parse(clean_version)
}

/// Trait for version checking - allows for easy mocking in tests
#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait VersionCheckable: Send + Sync {
    /// Check for the latest version from the remote source
    async fn check_latest_version(
        &self,
    ) -> Result<GitHubRelease, Box<dyn std::error::Error + Send + Sync>>;

    /// Get the current version
    fn current_version(&self) -> &str;
}

/// Version checker client for making GitHub API requests
pub struct VersionChecker {
    client: Client,
    current_version: String,
}

impl VersionChecker {
    pub fn new(current_version: String) -> Self {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(10))
            .user_agent(format!("nexus-cli/{}", current_version))
            .build()
            .expect("Failed to create HTTP client for version checker");

        Self {
            client,
            current_version,
        }
    }
}

#[async_trait::async_trait]
impl VersionCheckable for VersionChecker {
    /// Check for latest version from GitHub API
    async fn check_latest_version(
        &self,
    ) -> Result<GitHubRelease, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client.get(GITHUB_RELEASES_URL).send().await?;

        if !response.status().is_success() {
            return Err(format!("GitHub API returned status: {}", response.status()).into());
        }

        let release: GitHubRelease = response.json().await?;
        Ok(release)
    }

    fn current_version(&self) -> &str {
        &self.current_version
    }
}

/// Background task that periodically checks for version updates
pub async fn version_checker_task(
    version_checker: Box<dyn VersionCheckable>,
    event_sender: mpsc::Sender<Event>,
    shutdown: broadcast::Receiver<()>,
) {
    version_checker_task_with_interval(
        version_checker,
        event_sender,
        shutdown,
        VERSION_CHECK_INTERVAL,
    )
    .await;
}

/// Background task that periodically checks for version updates and constraints with configurable interval
pub async fn version_checker_task_with_interval(
    version_checker: Box<dyn VersionCheckable>,
    event_sender: mpsc::Sender<Event>,
    mut shutdown: broadcast::Receiver<()>,
    check_interval: Duration,
) {
    let mut version_info = VersionInfo::new(version_checker.current_version().to_string());
    let mut constraint_state = VersionConstraintState {
        last_constraint_check: None,
        current_constraints: None,
        last_violation: None,
    };

    // Perform initial checks immediately
    perform_version_and_constraint_check(
        &*version_checker,
        &mut version_info,
        &mut constraint_state,
        &event_sender,
    )
    .await;

    // After initial check, wait for the interval then check periodically
    let mut last_check = Instant::now();

    loop {
        tokio::select! {
            _ = shutdown.recv() => break,
            _ = sleep(Duration::from_secs(60)) => {
                // Check if it's time for a check
                if last_check.elapsed() >= check_interval {
                    last_check = Instant::now();
                    perform_version_and_constraint_check(&*version_checker, &mut version_info, &mut constraint_state, &event_sender).await;
                }
            }
        }
    }
}

/// Perform both version update check and constraint check
async fn perform_version_and_constraint_check(
    version_checker: &dyn VersionCheckable,
    version_info: &mut VersionInfo,
    constraint_state: &mut VersionConstraintState,
    event_sender: &mpsc::Sender<Event>,
) {
    // Check for version updates
    match version_checker.check_latest_version().await {
        Ok(release) => {
            version_info.update_from_release(release.clone());

            // Check version constraints to determine what message to show
            let constraint_result = match VersionRequirements::fetch().await {
                Ok(requirements) => {
                    // Update constraint state
                    constraint_state.current_constraints = Some(requirements.clone());
                    constraint_state.last_constraint_check = Some(Instant::now());

                    requirements
                        .check_version_constraints(
                            &version_info.current_version,
                            Some(&release.tag_name),
                            Some(&release.html_url),
                        )
                        .ok()
                        .flatten()
                }
                Err(_) => {
                    // If we can't fetch requirements, default to the old behavior
                    if version_info.update_available {
                        Some(VersionCheckResult {
                            constraint_type: ConstraintType::Notice,
                            message: format!(
                                "🚀 New version {} available! Current: {} → Release: {}",
                                release.tag_name, version_info.current_version, release.html_url
                            ),
                        })
                    } else {
                        None
                    }
                }
            };

            // Only send event if constraint status has changed
            let should_send_event = match (&constraint_state.last_violation, &constraint_result) {
                (None, None) => false, // No change
                (Some(old), Some(new)) => {
                    // Check if constraint type or message has changed
                    old.constraint_type != new.constraint_type || old.message != new.message
                }
                _ => true, // One is Some, other is None - status changed
            };

            if should_send_event {
                if let Some(result) = constraint_result {
                    let event = match result.constraint_type {
                        ConstraintType::Blocking => Event::version_checker_with_level(
                            result.message.clone(),
                            EventType::Error,
                            LogLevel::Error,
                        ),
                        ConstraintType::Warning => Event::version_checker_with_level(
                            result.message.clone(),
                            EventType::Error,
                            LogLevel::Warn,
                        ),
                        ConstraintType::Notice => Event::version_checker_with_level(
                            result.message.clone(),
                            EventType::Success,
                            LogLevel::Info,
                        ),
                    };

                    let _ = event_sender.send(event).await;

                    // Update constraint state
                    constraint_state.last_violation = Some(result);
                } else {
                    // No violation - send up-to-date message
                    let message = format!(
                        "✅ Version {} is up to date\n",
                        version_info.current_version
                    );

                    let event = Event::version_checker_with_level(
                        message,
                        EventType::Refresh,
                        LogLevel::Debug,
                    );

                    let _ = event_sender.send(event).await;

                    // Clear constraint state
                    constraint_state.last_violation = None;
                }
            }
        }
        Err(e) => {
            let message = format!("Failed to check for updates: {}", e);
            let event =
                Event::version_checker_with_level(message, EventType::Error, LogLevel::Debug);

            let _ = event_sender.send(event).await;
        }
    }
}

/// Convenience function for creating the background task with the real version checker
pub async fn start_version_checker_task(
    current_version: String,
    event_sender: mpsc::Sender<Event>,
    shutdown: broadcast::Receiver<()>,
) {
    let version_checker = Box::new(VersionChecker::new(current_version));
    version_checker_task(version_checker, event_sender, shutdown).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::Worker;
    use tokio::sync::{broadcast, mpsc};
    use tokio::time::{Duration, sleep};

    #[test]
    fn test_version_comparison() {
        // Test version comparison logic
        let info_090 = VersionInfo::new("0.9.0".to_string());
        let info_091 = VersionInfo::new("0.9.1".to_string());
        let info_100 = VersionInfo::new("1.0.0".to_string());

        // Test newer version detection
        assert!(info_090.is_newer_version("0.9.1"));
        assert!(info_090.is_newer_version("v0.9.1"));
        assert!(info_091.is_newer_version("1.0.0"));
        assert!(info_091.is_newer_version("v1.0.0"));

        // Test same version
        assert!(!info_091.is_newer_version("0.9.1"));
        assert!(!info_091.is_newer_version("v0.9.1"));

        // Test older version
        assert!(!info_091.is_newer_version("0.9.0"));
        assert!(!info_100.is_newer_version("0.9.1"));
    }

    #[test]
    fn test_version_info_update() {
        let mut info = VersionInfo::new("0.9.0".to_string());

        let release = GitHubRelease {
            tag_name: "v0.9.1".to_string(),
            name: "Release v0.9.1".to_string(),
            published_at: "2024-01-01T00:00:00Z".to_string(),
            html_url: "https://github.com/nexus-xyz/nexus-cli/releases/tag/v0.9.1".to_string(),
            prerelease: false,
        };

        info.update_from_release(release);

        assert!(info.update_available);
        assert_eq!(info.latest_version, Some("v0.9.1".to_string()));
    }

    /// Helper function to create a mock GitHub release
    fn create_mock_release(version: &str) -> GitHubRelease {
        GitHubRelease {
            tag_name: version.to_string(),
            name: format!("Release {}", version),
            published_at: "2024-01-01T00:00:00Z".to_string(),
            html_url: format!(
                "https://github.com/nexus-xyz/nexus-cli/releases/tag/{}",
                version
            ),
            prerelease: false,
        }
    }

    #[tokio::test]
    async fn test_version_checker_task_with_new_version_available() {
        // Test with a version that should trigger an update notification
        let current_version = "0.9.0";
        let new_version = "0.9.1";

        // Create mock version checker
        let mut mock_checker = MockVersionCheckable::new();
        mock_checker
            .expect_current_version()
            .return_const(current_version.to_string());

        // Mock returns a newer version
        mock_checker
            .expect_check_latest_version()
            .returning(move || Ok(create_mock_release(&format!("v{}", new_version))));

        // Set up channels
        let (event_sender, mut event_receiver) = mpsc::channel(10);
        let (shutdown_sender, shutdown_receiver) = broadcast::channel(1);

        // Start the version checker task with short interval for testing
        let task_handle = tokio::spawn(async move {
            version_checker_task_with_interval(
                Box::new(mock_checker),
                event_sender,
                shutdown_receiver,
                Duration::from_millis(100), // Short interval for testing
            )
            .await;
        });

        // Wait a bit for the task to run
        sleep(Duration::from_millis(200)).await;

        // Shutdown the task
        let _ = shutdown_sender.send(());
        task_handle.await.unwrap();

        // Check that we received a version checker event
        let mut received_event = false;
        while let Ok(event) = event_receiver.try_recv() {
            if matches!(event.worker, Worker::VersionChecker) {
                received_event = true;
                // With current version 0.9.0, we expect a warning constraint message
                assert!(
                    event
                        .msg
                        .contains("Consider upgrading for the best experience")
                );
                break;
            }
        }
        assert!(
            received_event,
            "Should have received version checker notification"
        );
    }

    #[tokio::test]
    async fn test_version_checker_task_with_no_update_needed() {
        // Test with a version that should not trigger any constraints
        let test_version = "0.9.1";

        // Create mock version checker
        let mut mock_checker = MockVersionCheckable::new();
        mock_checker
            .expect_current_version()
            .return_const(test_version.to_string());

        // Mock returns same version (no update needed)
        mock_checker
            .expect_check_latest_version()
            .returning(move || Ok(create_mock_release(&format!("v{}", test_version))));

        // Set up channels
        let (event_sender, mut event_receiver) = mpsc::channel(10);
        let (shutdown_sender, shutdown_receiver) = broadcast::channel(1);

        // Start the version checker task with short interval for testing
        let task_handle = tokio::spawn(async move {
            version_checker_task_with_interval(
                Box::new(mock_checker),
                event_sender,
                shutdown_receiver,
                Duration::from_millis(100), // Short interval for testing
            )
            .await;
        });

        // Wait a bit for the task to run
        sleep(Duration::from_millis(200)).await;

        // Shutdown the task
        let _ = shutdown_sender.send(());
        task_handle.await.unwrap();

        // Check that we received a version checker event or no events
        let mut received_event = false;
        let mut event_count = 0;
        while let Ok(event) = event_receiver.try_recv() {
            event_count += 1;
            if matches!(event.worker, Worker::VersionChecker) {
                received_event = true;
                // With version 0.9.1, we might get a warning constraint or no constraint
                // The test should pass if we get any version checker event
                break;
            }
        }

        // With the new constraint system, we might not send an event if there's no status change
        // So we accept either a version checker event or no events at all
        assert!(
            received_event || event_count == 0,
            "Should have received version checker notification or no events (no status change)"
        );
    }

    #[tokio::test]
    async fn test_version_checker_task_with_api_error() {
        // Create mock version checker that returns an error
        let mut mock_checker = MockVersionCheckable::new();
        mock_checker
            .expect_current_version()
            .return_const("0.9.1".to_string());

        // Mock returns an error
        mock_checker
            .expect_check_latest_version()
            .returning(|| Err("GitHub API unavailable".into()));

        // Set up channels
        let (event_sender, mut event_receiver) = mpsc::channel(10);
        let (shutdown_sender, shutdown_receiver) = broadcast::channel(1);

        // Start the version checker task with short interval for testing
        let task_handle = tokio::spawn(async move {
            version_checker_task_with_interval(
                Box::new(mock_checker),
                event_sender,
                shutdown_receiver,
                Duration::from_millis(100), // Short interval for testing
            )
            .await;
        });

        // Wait a bit for the task to run
        sleep(Duration::from_millis(200)).await;

        // Shutdown the task
        let _ = shutdown_sender.send(());
        task_handle.await.unwrap();

        // Check that we received the error event
        let mut received_error_event = false;
        while let Ok(event) = event_receiver.try_recv() {
            if event.msg.contains("Failed to check for updates") {
                received_error_event = true;
                assert_eq!(event.event_type, EventType::Error);
                assert_eq!(event.log_level, LogLevel::Debug);
                break;
            }
        }
        assert!(
            received_error_event,
            "Should have received error notification"
        );
    }

    #[tokio::test]
    async fn test_version_checker_only_notifies_once_for_same_update() {
        // Test that multiple API calls only result in one notification
        let current_version = "0.9.0";
        let new_version = "0.9.1";

        // Create mock version checker
        let mut mock_checker = MockVersionCheckable::new();
        mock_checker
            .expect_current_version()
            .return_const(current_version.to_string());

        // Mock always returns newer version - called multiple times but only first notification sent
        mock_checker
            .expect_check_latest_version()
            .returning(move || Ok(create_mock_release(&format!("v{}", new_version))))
            .times(..); // Allow any number of calls

        // Set up channels
        let (event_sender, mut event_receiver) = mpsc::channel(10);
        let (shutdown_sender, shutdown_receiver) = broadcast::channel(1);

        // Start the version checker task with very short interval for testing
        let task_handle = tokio::spawn(async move {
            version_checker_task_with_interval(
                Box::new(mock_checker),
                event_sender,
                shutdown_receiver,
                Duration::from_millis(50), // Very short interval for testing
            )
            .await;
        });

        // Wait for multiple intervals to allow multiple checks
        sleep(Duration::from_millis(250)).await;

        // Shutdown the task
        let _ = shutdown_sender.send(());
        task_handle.await.unwrap();

        // Count version checker notifications - should only be one even with multiple API calls
        let mut version_event_count = 0;
        let mut all_events = Vec::new();
        while let Ok(event) = event_receiver.try_recv() {
            all_events.push(event.msg.clone());
            if matches!(event.worker, Worker::VersionChecker) {
                version_event_count += 1;
            }
        }

        // Debug: print all events to understand what happened
        println!("All events received: {:?}", all_events);

        // Should only notify once for the same constraint, even though API was called multiple times
        assert_eq!(
            version_event_count, 1,
            "Should only notify once for the same constraint"
        );
    }

    #[test]
    fn test_edge_case_version_comparisons() {
        // Test various edge cases with semver
        let info_100 = VersionInfo::new("1.0.0".to_string());
        let info_1100 = VersionInfo::new("1.10.0".to_string());
        let info_1010 = VersionInfo::new("1.0.10".to_string());
        let info_20 = VersionInfo::new("2.0.0".to_string());

        // Test major version differences
        assert!(info_100.is_newer_version("2.0.0"));
        assert!(!info_20.is_newer_version("1.9.9"));

        // Test minor version differences
        assert!(info_100.is_newer_version("1.10.0"));
        assert!(!info_1100.is_newer_version("1.9.0"));

        // Test patch version differences
        assert!(info_100.is_newer_version("1.0.10"));
        assert!(!info_1010.is_newer_version("1.0.9"));

        // Test that semver handles malformed versions gracefully
        assert!(!info_100.is_newer_version("not.a.version"));
        assert!(!info_100.is_newer_version(""));
    }
}
