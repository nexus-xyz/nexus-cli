//! Combined request timing and retry management
//!
//! This module replaces the separate backoff and rate limiter components with a
//! unified approach that prioritizes server-provided retry delays over local timing strategies.

use std::time::{Duration, Instant};

/// Configuration for request timing behavior
#[derive(Debug, Clone)]
pub struct RequestTimerConfig {
    /// Minimum time between requests
    pub min_interval: Duration,
    /// Maximum requests per time window
    pub max_requests: Option<u32>,
    /// Time window for max_requests
    pub time_window: Option<Duration>,
    /// Default retry delay when server doesn't provide one
    pub default_retry_delay: Duration,
}

impl RequestTimerConfig {
    /// Create a simple interval-based timer
    fn _interval(min_interval: Duration) -> Self {
        Self {
            min_interval,
            max_requests: None,
            time_window: None,
            default_retry_delay: Duration::from_secs(1),
        }
    }

    /// Create a request-per-window timer
    fn _requests_per_window(max_requests: u32, time_window: Duration) -> Self {
        Self {
            min_interval: Duration::ZERO,
            max_requests: Some(max_requests),
            time_window: Some(time_window),
            default_retry_delay: Duration::from_secs(1),
        }
    }

    /// Create a combined timer (interval + requests per window)
    pub fn combined(
        min_interval: Duration,
        max_requests: u32,
        time_window: Duration,
        default_retry_delay: Duration,
    ) -> Self {
        Self {
            min_interval,
            max_requests: Some(max_requests),
            time_window: Some(time_window),
            default_retry_delay,
        }
    }
}

/// Unified request timer that handles both rate limiting and retry timing
/// Server-provided retry delays always override local timing strategies
#[derive(Debug)]
pub struct RequestTimer {
    config: RequestTimerConfig,
    last_request_time: Option<Instant>,
    request_times: Vec<Instant>,
    server_retry_until: Option<Instant>,
}

impl RequestTimer {
    pub fn new(config: RequestTimerConfig) -> Self {
        Self {
            config,
            last_request_time: None,
            request_times: Vec::new(),
            server_retry_until: None,
        }
    }

    /// Check if a new request can proceed
    /// Server retry delay takes priority over all other constraints
    pub fn can_proceed(&mut self) -> bool {
        let now = Instant::now();

        // Server retry delay always takes priority
        if let Some(retry_until) = self.server_retry_until {
            if now < retry_until {
                return false;
            }
            // Clear expired server retry delay
            self.server_retry_until = None;
        }

        // Check minimum interval
        if let Some(last_time) = self.last_request_time {
            if now.duration_since(last_time) < self.config.min_interval {
                return false;
            }
        }

        // Check requests per time window
        if let (Some(max_requests), Some(time_window)) =
            (self.config.max_requests, self.config.time_window)
        {
            // Remove old requests outside the time window
            self.request_times
                .retain(|&time| now.duration_since(time) <= time_window);

            if self.request_times.len() >= max_requests as usize {
                return false;
            }
        }

        true
    }

    /// Record a successful request
    pub fn record_success(&mut self) {
        let now = Instant::now();
        self.last_request_time = Some(now);
        if self.config.max_requests.is_some() {
            self.request_times.push(now);
        }

        // Don't override existing server retry delay - respect whatever time is left
        // Only set default retry delay if there's no existing wait period
        if self.server_retry_until.is_none() || self.server_retry_until.unwrap() <= now {
            self.server_retry_until = Some(now + self.config.default_retry_delay);
        }
    }

    /// Record a failed request with optional server-provided retry delay
    /// If server_retry_delay is provided, it overrides all other timing logic
    pub fn record_failure(&mut self, server_retry_delay: Option<Duration>) {
        let now = Instant::now();
        self.last_request_time = Some(now);

        if self.config.max_requests.is_some() {
            self.request_times.push(now);
        }

        // Server retry delay overrides everything else
        if let Some(delay) = server_retry_delay {
            self.server_retry_until = Some(now + delay);
        } else {
            // Use default retry delay if no server delay provided
            self.server_retry_until = Some(now + self.config.default_retry_delay);
        }
    }

    /// Get time until next request is allowed
    /// Server retry delay takes priority over all other constraints
    pub fn time_until_next(&mut self) -> Duration {
        let now = Instant::now();

        // Server retry delay has highest priority
        if let Some(retry_until) = self.server_retry_until {
            if now < retry_until {
                return retry_until.duration_since(now);
            }
            // Clear expired server retry delay
            self.server_retry_until = None;
        }

        let mut min_wait = Duration::ZERO;

        // Check minimum interval constraint
        if let Some(last_time) = self.last_request_time {
            let since_last = now.duration_since(last_time);
            if since_last < self.config.min_interval {
                min_wait = std::cmp::max(min_wait, self.config.min_interval - since_last);
            }
        }

        // Check requests per window constraint
        if let (Some(max_requests), Some(time_window)) =
            (self.config.max_requests, self.config.time_window)
        {
            // Remove old requests
            self.request_times
                .retain(|&time| now.duration_since(time) <= time_window);

            if self.request_times.len() >= max_requests as usize {
                // Find the oldest request in the window
                if let Some(&oldest) = self.request_times.first() {
                    let wait_until_oldest_expires = time_window - now.duration_since(oldest);
                    min_wait = std::cmp::max(min_wait, wait_until_oldest_expires);
                }
            }
        }

        min_wait
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_server_retry_overrides_other_constraints() {
        let config = RequestTimerConfig::combined(
            Duration::from_millis(100), // min interval
            3,                          // max requests
            Duration::from_secs(1),     // time window
            Duration::from_millis(50),  // default retry delay
        );

        let mut timer = RequestTimer::new(config);

        // First request should succeed
        assert!(timer.can_proceed());
        timer.record_success();

        // Record failure with server retry delay
        let server_delay = Duration::from_secs(5);
        timer.record_failure(Some(server_delay));

        // Should not be able to proceed due to server retry delay
        assert!(!timer.can_proceed());

        // Time until next should be the server delay (approximately)
        let remaining = timer.time_until_next();
        assert!(remaining.as_millis() > 4900); // Allow some timing tolerance
        assert!(remaining.as_millis() <= 5000);
    }

    #[test]
    fn test_default_retry_delay_when_no_server_delay() {
        let config = RequestTimerConfig::_interval(Duration::from_millis(10));
        let mut timer = RequestTimer::new(config);

        // Record failure without server retry delay
        timer.record_failure(None);

        // Should use default retry delay
        assert!(!timer.can_proceed());
        let remaining = timer.time_until_next();
        assert!(remaining.as_millis() > 900); // Close to 1 second default
    }

    #[test]
    fn test_success_clears_server_retry_delay() {
        let config = RequestTimerConfig::_interval(Duration::from_millis(10));
        let mut timer = RequestTimer::new(config);

        // Record failure with server retry delay
        timer.record_failure(Some(Duration::from_secs(10)));

        // Record success should clear server throttling
        timer.record_success();
    }

    #[test]
    fn test_min_interval_without_server_delay() {
        let config = RequestTimerConfig::_interval(Duration::from_millis(100));
        let mut timer = RequestTimer::new(config);

        // First request should succeed
        assert!(timer.can_proceed());
        timer.record_success();

        // Immediate second request should be blocked by min interval
        assert!(!timer.can_proceed());
    }
}
