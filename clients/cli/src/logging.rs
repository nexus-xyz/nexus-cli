use crate::error_classifier::LogLevel;
use std::env;

pub fn get_rust_log_level() -> LogLevel {
    let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    parse_rust_log_level(&rust_log)
}

pub fn parse_rust_log_level(rust_log: &str) -> LogLevel {
    // Handle common RUST_LOG formats
    let level_str = rust_log
        .split(',')
        .next()
        .unwrap_or(rust_log)
        .split('=')
        .next_back()
        .unwrap_or(rust_log)
        .to_lowercase();

    match level_str.as_str() {
        "trace" => LogLevel::Trace,
        "debug" => LogLevel::Debug,
        "info" => LogLevel::Info,
        "warn" | "warning" => LogLevel::Warn,
        "error" => LogLevel::Error,
        _ => LogLevel::Info, // Default to info if parsing fails
    }
}

pub fn should_log(event_level: LogLevel, threshold: LogLevel) -> bool {
    event_level >= threshold
}

pub fn should_log_with_env(event_level: LogLevel) -> bool {
    let threshold = get_rust_log_level();
    should_log(event_level, threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rust_log_level() {
        assert_eq!(parse_rust_log_level("debug"), LogLevel::Debug);
        assert_eq!(parse_rust_log_level("info"), LogLevel::Info);
        assert_eq!(parse_rust_log_level("warn"), LogLevel::Warn);
        assert_eq!(parse_rust_log_level("error"), LogLevel::Error);
        assert_eq!(parse_rust_log_level("trace"), LogLevel::Trace);

        // Test with module-specific formats
        assert_eq!(parse_rust_log_level("nexus_cli=debug"), LogLevel::Debug);
        assert_eq!(
            parse_rust_log_level("nexus_cli=debug,hyper=info"),
            LogLevel::Debug
        );

        // Test default
        assert_eq!(parse_rust_log_level("invalid"), LogLevel::Info);
    }

    #[test]
    fn test_should_log() {
        assert!(should_log(LogLevel::Error, LogLevel::Debug));
        assert!(should_log(LogLevel::Warn, LogLevel::Warn));
        assert!(!should_log(LogLevel::Debug, LogLevel::Error));
        assert!(!should_log(LogLevel::Info, LogLevel::Error));
    }
}
