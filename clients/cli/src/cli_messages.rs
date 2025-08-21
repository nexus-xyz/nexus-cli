//! CLI command messaging system
//!
//! This module provides consistent messaging for CLI commands like registration,
//! logout, and other command-line operations.

/// Print CLI command info message (for registration, logout, etc.)
pub fn print_info(title: &str, details: &str) {
    print!("\x1b[1;33m[INFO]\x1b[0m {}", title);
    if !details.is_empty() {
        println!("\t {}", details);
    } else {
        println!();
    }
}

/// Print CLI command warn message
pub fn print_warn(title: &str, details: &str) {
    print!("\x1b[1;91m[WARN]\x1b[0m {}", title);
    if !details.is_empty() {
        println!("\t {}", details);
    } else {
        println!();
    }
}

/// Print CLI command error
pub fn print_error(title: &str, details: Option<&str>) {
    println!("\x1b[1;31m[ERROR]\x1b[0m {}", title);
    if let Some(details) = details {
        println!("\x1b[1;31m[ERROR]\x1b[0m Details: {}", details);
    }
}

/// Print CLI command success
pub fn print_success(title: &str, details: &str) {
    print!("\x1b[1;32m[SUCCESS]\x1b[0m {}", title);
    if !details.is_empty() {
        println!("\t {}", details);
    } else {
        println!();
    }
}

/// Macro for backward compatibility with existing print_cmd_info! usage
#[macro_export]
macro_rules! print_cmd_info {
    ($title:expr, $($details:tt)*) => {
        $crate::cli_messages::print_info($title, &format!($($details)*))
    };
}

/// Macro for print_cmd_warn! usage
#[macro_export]
macro_rules! print_cmd_warn {
    ($title:expr, $($details:tt)*) => {
        $crate::cli_messages::print_warn($title, &format!($($details)*))
    };
}

/// Macro for CLI errors
#[macro_export]
macro_rules! print_cmd_error {
    ($title:expr) => {
        $crate::cli_messages::print_error($title, None)
    };
    ($title:expr, $details:expr) => {
        $crate::cli_messages::print_error($title, Some($details))
    };
}

/// Macro for CLI success messages
#[macro_export]
macro_rules! print_cmd_success {
    ($title:expr, $($details:tt)*) => {
        $crate::cli_messages::print_success($title, &format!($($details)*))
    };
}
