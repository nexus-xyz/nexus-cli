//! Version management and validation with improved error messages
use super::{ConstraintType, VersionRequirements};
use std::error::Error;
use std::time::Duration;

/// Validates version requirements before application startup
pub async fn validate_version_requirements() -> Result<(), Box<dyn Error>> {
    // Try fetching requirements with retries
    let requirements = match retry_fetch_requirements().await {
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

/// Retry fetching version requirements with exactly 3 attempts
async fn retry_fetch_requirements() -> Result<VersionRequirements, Box<dyn Error>> {
    for attempt in 1..=3 {
        match VersionRequirements::fetch().await {
            Ok(requirements) => return Ok(requirements),
            Err(e) => {
                if attempt == 3 {
                    return Err(e);
                }
                eprintln!("Version check failed (attempt {}/3), retrying in {} seconds...", 
                    attempt, attempt * 2);
                tokio::time::sleep(Duration::from_secs(attempt as u64 * 2)).await;
            }
        }
    }
    unreachable!()
}

/// Provides user-friendly error messages for fetch failures
fn handle_fetch_error(error: &Box<dyn Error>) {
    let error_str = error.to_string();
    
    eprintln!("❌ Unable to verify CLI version requirements after multiple attempts\n");
    
    if error_str.contains("error sending request") || error_str.contains("Failed to fetch from all sources") {
        eprintln!("Network connectivity issue detected.\n");
        eprintln!("Troubleshooting steps:");
        eprintln!("1. Check your internet connection");
        eprintln!("2. Verify these domains are accessible:");
        eprintln!("   • cli.nexus.xyz");
        eprintln!("   • raw.githubusercontent.com");
        eprintln!("3. Try the following commands:");
        eprintln!("   curl -v https://cli.nexus.xyz/version.json");
        eprintln!("   ping cli.nexus.xyz\n");
    } else if error_str.contains("timeout") {
        eprintln!("The request timed out. This could indicate:");
        eprintln!("  • Slow or unstable internet connection");
        eprintln!("  • High latency to the Nexus servers");
        eprintln!("  • Temporary server issues\n");
        
        eprintln!("Please try:");
        eprintln!("  • Waiting a few moments and trying again");
        eprintln!("  • Checking your internet speed and stability");
        eprintln!("  • Using a different network if available\n");
    } else if error_str.contains("certificate") || error_str.contains("SSL") || error_str.contains("TLS") {
        eprintln!("There's an SSL/TLS certificate issue. Please check:");
        eprintln!("  • Your system date and time are correct");
        eprintln!("  • Your CA certificates are up to date");
        eprintln!("  • You're not on a network performing SSL interception\n");
        
        eprintln!("On Linux, you can update certificates with:");
        eprintln!("  sudo apt-get update && sudo apt-get install ca-certificates");
        eprintln!("  OR");
        eprintln!("  sudo yum install ca-certificates\n");
    } else if error_str.contains("DNS") || error_str.contains("resolve") {
        eprintln!("DNS resolution failed. Please check:");
        eprintln!("  • Your DNS settings (try using 8.8.8.8 or 1.1.1.1)");
        eprintln!("  • Your network connection");
        eprintln!("  • Try flushing your DNS cache\n");
        
        eprintln!("To flush DNS cache:");
        eprintln!("  • Linux: sudo systemd-resolve --flush-caches");
        eprintln!("  • macOS: sudo dscacheutil -flushcache");
        eprintln!("  • Windows: ipconfig /flushdns\n");
    } else {
        // Generic error with details
        eprintln!("Error details: {}\n", error_str);
        eprintln!("Common solutions:");
        eprintln!("  • Check your internet connection");
        eprintln!("  • Try again in a few moments");
        eprintln!("  • Check if Nexus services are operational\n");
    }
    
    eprintln!("If this issue persists after trying the above solutions:");
    eprintln!("  • Check known issues: https://github.com/nexus-xyz/nexus-cli/issues");
    eprintln!("  • Report a bug: https://github.com/nexus-xyz/nexus-cli/issues/new");
    eprintln!("  • Include this error message and your environment details");
}

/// Handles different types of version constraint violations
fn handle_version_violation(constraint_type: &ConstraintType, message: &str) {
    match constraint_type {
        ConstraintType::Blocking => {
            eprintln!("❌ Version requirement not met\n");
            eprintln!("{}\n", message);
            eprintln!("To resolve this issue:");
            eprintln!("  • Update your CLI: nexus update");
            eprintln!("  • Or manually download the latest version from:");
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