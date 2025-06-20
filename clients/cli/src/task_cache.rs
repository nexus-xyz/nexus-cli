//! Cache for recently used task IDs.

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Thread-safe queue of most recent task IDs (bounded).
#[derive(Clone, Debug)]
pub struct TaskCache {
    capacity: usize,
    inner: Arc<Mutex<VecDeque<String>>>,
}

impl TaskCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
        }
    }

    /// Returns true if the task ID is already in the queue.
    pub async fn contains(&self, task_id: &str) -> bool {
        let queue = self.inner.lock().await;
        queue.iter().any(|id| id == task_id)
    }

    /// Appends a task ID to the queue, evicting the oldest if full.
    pub async fn insert(&self, task_id: String) {
        let mut queue = self.inner.lock().await;
        if queue.contains(&task_id) {
            return;
        }
        if queue.len() == self.capacity {
            queue.pop_front();
        }
        queue.push_back(task_id);
    }
}
