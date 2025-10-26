pub mod cli_consts {
    //! Prover Configuration Constants
    //!
    //! This module contains all configuration constants for the prover system,
    //! organized by functional area for clarity and maintainability.

    // =============================================================================
    // QUEUE CONFIGURATION
    // =============================================================================
    // All queue sizes are chosen to be larger than the API page size (currently 50)
    // to provide adequate buffering while preventing excessive memory usage.

    /// The maximum number of events to keep in the activity logs.
    pub const MAX_ACTIVITY_LOGS: usize = 100;

    /// Maximum number of event buffer size for worker threads
    pub const EVENT_QUEUE_SIZE: usize = 100;

    // =============================================================================
    // PROVING CONFIGURATIONS
    // =============================================================================

    /// Subprocess error code likely indicating an OOM error
    pub const SUBPROCESS_SUSPECTED_OOM_CODE: i32 = 137;

    /// Subprocess error code indicating an internal failure of the proving
    pub const SUBPROCESS_INTERNAL_ERROR_CODE: i32 = 3;

    /// "Reasonable" generic projection task memory requirement.
    pub const PROJECTED_MEMORY_REQUIREMENT: u64 = 2147483648; // 2gb

    // =============================================================================
    // DIFFICULTY CONFIGURATION
    // =============================================================================

    /// Task difficulty system configuration
    pub mod difficulty {
        /// Time threshold for auto-promotion (seconds)
        /// Tasks completing faster than this will promote to next difficulty level
        pub const PROMOTION_THRESHOLD_SECS: u64 = 7 * 60; // 7 minutes
    }

    // =============================================================================
    // NETWORK CONFIGURATION
    // =============================================================================

    /// Task fetching backoff configuration
    pub mod task_fetching {
        use std::time::Duration;

        /// Initial delay before retrying failed task fetch (milliseconds)
        /// Set to 2 minutes to align with server task creation frequency
        pub const INITIAL_BACKOFF_MS: u64 = 120_000;
        /// Maximum number of retry attempts for task fetching
        pub const MAX_RETRIES: u32 = 2;

        /// Minimum interval between task fetch requests (milliseconds)
        /// Set to 2 minutes to align with server task creation frequency
        pub const RATE_LIMIT_INTERVAL_MS: u64 = 120_000;

        /// Helper function to get initial backoff duration
        pub const fn initial_backoff() -> Duration {
            Duration::from_millis(INITIAL_BACKOFF_MS)
        }

        /// Helper function to get rate limit interval
        pub const fn rate_limit_interval() -> Duration {
            Duration::from_millis(RATE_LIMIT_INTERVAL_MS)
        }
    }

    /// Proof submission backoff configuration
    pub mod proof_submission {
        use std::time::Duration;

        /// Initial delay before retrying failed proof submission (milliseconds)
        /// More aggressive than task fetching since submissions are critical
        pub const INITIAL_BACKOFF_MS: u64 = 1000; // 1 second

        /// Maximum number of retry attempts for proof submission
        /// More retries since submissions are critical
        pub const MAX_RETRIES: u32 = 5;

        /// Minimum interval between submission requests (milliseconds)
        /// Less restrictive than task fetching
        pub const RATE_LIMIT_INTERVAL_MS: u64 = 100;

        /// Helper function to get initial backoff duration
        pub const fn initial_backoff() -> Duration {
            Duration::from_millis(INITIAL_BACKOFF_MS)
        }

        /// Helper function to get rate limit interval
        pub const fn rate_limit_interval() -> Duration {
            Duration::from_millis(RATE_LIMIT_INTERVAL_MS)
        }
    }

    /// Advanced rate limiting configuration
    pub mod rate_limiting {
        use std::time::Duration;

        /// Maximum requests per time window for task fetching
        pub const TASK_FETCH_MAX_REQUESTS_PER_WINDOW: u32 = 60;

        /// Time window duration for task fetching rate limiting (milliseconds)
        pub const TASK_FETCH_WINDOW_MS: u64 = 60_000; // 1 minute

        /// Maximum requests per time window for proof submission
        pub const SUBMISSION_MAX_REQUESTS_PER_WINDOW: u32 = 100;

        /// Time window duration for proof submission rate limiting (milliseconds)
        pub const SUBMISSION_WINDOW_MS: u64 = 60_000; // 1 minute

        /// Helper function to get task fetch time window
        pub const fn task_fetch_window() -> Duration {
            Duration::from_millis(TASK_FETCH_WINDOW_MS)
        }

        /// Helper function to get submission time window
        pub const fn submission_window() -> Duration {
            Duration::from_millis(SUBMISSION_WINDOW_MS)
        }

        /// Extra delay to add on top of server-provided retry delays
        pub const EXTRA_RETRY_DELAY_SECS: u64 = 10;

        /// Helper function to get the extra retry delay
        pub const fn extra_retry_delay() -> Duration {
            Duration::from_secs(EXTRA_RETRY_DELAY_SECS)
        }
    }
}
