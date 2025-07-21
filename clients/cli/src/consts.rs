pub mod prover {
    // Queue sizes. Chosen to be larger than the tasks API page size (currently, 50)
    pub const TASK_QUEUE_SIZE: usize = 100;
    pub const EVENT_QUEUE_SIZE: usize = 100;
    pub const RESULT_QUEUE_SIZE: usize = 100;

    // Task fetching thresholds
    pub const BATCH_SIZE: usize = TASK_QUEUE_SIZE / 5; // Fetch this many tasks at once
    pub const LOW_WATER_MARK: usize = TASK_QUEUE_SIZE / 4; // Fetch new tasks when queue drops below this
    pub const MAX_404S_BEFORE_GIVING_UP: usize = 5; // Allow several 404s before stopping batch fetch
    pub const BACKOFF_DURATION: u64 = 120000; // 120 seconds
    pub const QUEUE_LOG_INTERVAL: u64 = 60000; // 1 minute

    /// How long a task ID remains in the duplicate-prevention cache before expiring.
    pub const CACHE_EXPIRATION: u64 = 300000; // 5 minutes
}
