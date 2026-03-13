//! Version management and validation with improved error messages
use super::{ConstraintType, VersionRequirements};
use std::error::Error;

/// Validates version requirements before application startup
pub async fn validate_version_requirements() -> Result<(), Box<dyn Error>> {
    // Single attempt since VersionRequirements::fetch already tries multiple hostnames
    let requirements = match VersionRequirements::fetch().await {
        Ok(requirements) => requirements,
        Err(e) => {
            handle_fetch_error(&e);
            std::process::exit(1);
        }
    };

    let current_version = env!("CARGO_PKG_VERSION");

    // Early OFAC block from server-provided list, if present
    let country = crate::orchestrator::client::detect_country_once().await;

    // Restriction check is against keys; printed names come from non-null values
    if requirements
        .ofac_country_names
        .keys()
        .any(|c| c.eq_ignore_ascii_case(&country))
    {
        let display_name = requirements
            .ofac_country_names
            .get(&country)
            .and_then(|v| v.clone())
            .unwrap_or_else(|| country.clone());
        eprintln!(
            "Due to OFAC regulations, this service is not available in {}.\nSee https://nexus.xyz/terms-of-use for more information.",
            display_name
        );
        std::process::exit(1);
    }

    match requirements.check_version_constraints(current_version, None, None) {
        Ok(Some(violation)) => {
            handle_version_violation(&violation.constraint_type, &violation.message);
        }
        Ok(None) => {
            // No violations found, continue normally
        }
        Err(e) => {
            eprintln!("❌ Failed to parse version requirements: {}", e);
            eprintln!(
                "If this issue persists, please file a bug report at: https://github.com/nexus-xyz/nexus-cli/issues/new"
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Provides user-friendly error messages for fetch failures
fn handle_fetch_error(error: &dyn Error) {
    eprintln!("❌ Unable to verify CLI version requirements");
    eprintln!("Reason: {}", error);
    eprintln!("\nPlease check your internet connection and try again.");
    eprintln!("If the issue persists, report it at: https://github.com/nexus-xyz/nexus-cli/issues");
}

/// Handles different types of version constraint violations
fn handle_version_violation(constraint_type: &ConstraintType, message: &str) {
    match constraint_type {
        ConstraintType::Blocking => {
            eprintln!("❌ Version requirement not met\n");
            eprintln!("{}\n", message);
            eprintln!("To resolve this issue:");
            eprintln!("  • Download the latest version from:");
            eprintln!("    https://github.com/nexus-xyz/nexus-cli/releases");
            std::process::exit(1);
        }
        ConstraintType::Warning => {
            eprintln!("⚠️  Version Warning");
            eprintln!("{}", message);
            eprintln!("Consider updating your CLI for the best experience.\n");
        }
        ConstraintType::Notice => {
            eprintln!("ℹ️  Notice");
            eprintln!("{}\n", message);
        }
    }
}
