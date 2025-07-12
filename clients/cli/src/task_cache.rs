//! Cache for recently used task IDs.

use crate::consts::prover::CACHE_EXPIRATION;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Thread-safe queue of most recent task IDs (bounded).
#[derive(Clone, Debug)]
pub struct TaskCache {
    capacity: usize,
    inner: Arc<Mutex<VecDeque<(String, Instant)>>>,
}

impl TaskCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
        }
    }

    /// Prune expired tasks from the cache.
    async fn prune_expired(&self) {
        let mut queue = self.inner.lock().await;
        queue
            .retain(|(_, timestamp)| timestamp.elapsed() < Duration::from_millis(CACHE_EXPIRATION));
    }

    /// Returns true if the task ID is already in the queue.
    pub async fn contains(&self, task_id: &str) -> bool {
        self.prune_expired().await;

        let queue = self.inner.lock().await;
        queue.iter().any(|(id, _)| id == task_id)
    }

    /// Appends a task ID to the queue, evicting the oldest if full.
    pub async fn insert(&self, task_id: String) {
        self.prune_expired().await;

        if self.contains(&task_id).await {
            return;
        }

        let mut queue = self.inner.lock().await;
        if queue.len() == self.capacity {
            queue.pop_front();
        }

        queue.push_back((task_id, Instant::now()));
    }
}
