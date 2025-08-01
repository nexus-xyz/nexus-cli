pub mod prover {
    //! Prover Configuration Constants
    //!
    //! This module contains all configuration constants for the prover system,
    //! organized by functional area for clarity and maintainability.

    // =============================================================================
    // QUEUE CONFIGURATION
    // =============================================================================
    // All queue sizes are chosen to be larger than the API page size (currently 50)
    // to provide adequate buffering while preventing excessive memory usage.

    /// Maximum number of tasks that can be queued for processing
    pub const TASK_QUEUE_SIZE: usize = 100;

    /// Maximum number of events that can be queued for UI updates
    pub const EVENT_QUEUE_SIZE: usize = 100;

    /// Maximum number of proof results that can be queued for submission
    pub const RESULT_QUEUE_SIZE: usize = 100;

    // =============================================================================
    // TASK FETCHING BEHAVIOR
    // =============================================================================

    /// Minimum queue level that triggers new task fetching
    /// When task queue drops below this threshold, fetch new tasks
    pub const LOW_WATER_MARK: usize = 1;

    // =============================================================================
    // TIMING AND BACKOFF CONFIGURATION
    // =============================================================================

    /// Default backoff duration when retrying failed operations (milliseconds)
    /// Set to 2 minutes to balance responsiveness with server load
    pub const BACKOFF_DURATION: u64 = 120_000; // 2 minutes

    // =============================================================================
    // CACHE MANAGEMENT
    // =============================================================================

    /// Duration to keep task IDs in duplicate-prevention cache (milliseconds)
    /// Long enough to prevent immediate re-processing, short enough to allow
    /// eventual retry of legitimately failed tasks
    pub const CACHE_EXPIRATION: u64 = 300_000; // 5 minutes

    // =============================================================================
    // COMPUTED CONSTANTS
    // =============================================================================

    /// Maximum number of completed tasks to track (prevents memory growth)
    /// Set to 5x the task queue size to provide adequate duplicate detection
    pub const MAX_COMPLETED_TASKS: usize = TASK_QUEUE_SIZE * 5;
}
