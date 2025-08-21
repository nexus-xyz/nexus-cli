//! System metrics collection and display.

use std::time::Instant;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// System metrics for display in the dashboard.
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// CPU usage percentage (0.0 to 100.0).
    pub cpu_percent: f32,
    /// Current process RAM usage in bytes.
    pub ram_bytes: u64,
    /// Peak process RAM usage in bytes since startup.
    pub peak_ram_bytes: u64,
    /// Total system RAM in bytes.
    pub total_ram_bytes: u64,
    /// Last time CPU was updated for proper refresh timing
    pub last_cpu_update: Option<Instant>,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            ram_bytes: 0,
            peak_ram_bytes: 0,
            total_ram_bytes: {
                let mut sys = System::new();
                sys.refresh_memory();
                sys.total_memory()
            },
            last_cpu_update: None,
        }
    }
}

impl SystemMetrics {
    /// Update metrics from system information, tracking peak memory over time.
    /// Uses proper CPU refresh timing according to sysinfo documentation.
    pub fn update(
        sysinfo: &mut System,
        previous_peak: u64,
        previous_metrics: Option<&SystemMetrics>,
    ) -> Self {
        let now = Instant::now();

        let current_pid = Pid::from(std::process::id() as usize);
        let mut cpu_total = 0.0;
        let mut ram_total = 0;

        // Check if enough time has passed for accurate CPU measurement
        let should_update_cpu = if let Some(prev) = previous_metrics {
            if let Some(last_update) = prev.last_cpu_update {
                now.duration_since(last_update) >= sysinfo::MINIMUM_CPU_UPDATE_INTERVAL
            } else {
                true // First time, always update
            }
        } else {
            true // No previous metrics, always update
        };

        let last_cpu_update = if should_update_cpu {
            // Refresh CPU usage and processes according to sysinfo best practices
            sysinfo.refresh_cpu_usage(); // Essential for CPU usage calculation
            // Refresh ALL processes to include subprocesses during proving
            sysinfo.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true, // Refresh exact processes
                ProcessRefreshKind::nothing().with_cpu().with_memory(),
            );
            Some(now)
        } else {
            // Still refresh ALL processes for memory tracking (including subprocesses)
            sysinfo.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                ProcessRefreshKind::nothing().with_memory(),
            );
            // Keep previous update time
            previous_metrics.and_then(|m| m.last_cpu_update)
        };

        // Get metrics for current process (both CPU and RAM)
        if let Some(process) = sysinfo.process(current_pid) {
            cpu_total = if should_update_cpu {
                process.cpu_usage()
            } else {
                // Use previous CPU value if not updating
                previous_metrics.map(|m| m.cpu_percent).unwrap_or(0.0)
            };
            // Use current process memory as base
            ram_total = process.memory();
        }

        // Include CPU and memory from nexus proving subprocesses
        for process in sysinfo.processes().values() {
            if process.parent() == Some(current_pid) {
                let process_name = process.name().to_string_lossy().to_lowercase();
                // Include child processes that are nexus-related (proving subprocesses)
                if process_name.contains("nexus") {
                    ram_total += process.memory();
                    if should_update_cpu {
                        cpu_total += process.cpu_usage(); // Add subprocess CPU usage!
                    }
                }
            }
        }

        // Track peak process RAM usage over application lifetime
        let peak_ram = previous_peak.max(ram_total);

        Self {
            cpu_percent: cpu_total,
            ram_bytes: ram_total,
            peak_ram_bytes: peak_ram,
            total_ram_bytes: sysinfo.total_memory(),
            last_cpu_update,
        }
    }

    /// Get RAM usage as a ratio (0.0 to 1.0).
    pub fn ram_ratio(&self) -> f64 {
        if self.total_ram_bytes == 0 {
            0.0
        } else {
            (self.ram_bytes as f64) / (self.total_ram_bytes as f64)
        }
    }

    /// Get peak RAM usage as a ratio (0.0 to 1.0).
    pub fn peak_ram_ratio(&self) -> f64 {
        if self.total_ram_bytes == 0 {
            0.0
        } else {
            (self.peak_ram_bytes as f64) / (self.total_ram_bytes as f64)
        }
    }

    /// Format RAM usage as human-readable string.
    pub fn format_ram(&self) -> String {
        let mb = self.ram_bytes as f64 / (1024.0 * 1024.0);
        if mb >= 1024.0 {
            format!("{:.1} GB", mb / 1024.0)
        } else {
            format!("{:.1} MB", mb)
        }
    }

    /// Format peak RAM usage as human-readable string.
    pub fn format_peak_ram(&self) -> String {
        let mb = self.peak_ram_bytes as f64 / (1024.0 * 1024.0);
        if mb >= 1024.0 {
            format!("{:.1} GB", mb / 1024.0)
        } else {
            format!("{:.1} MB", mb)
        }
    }

    /// Get CPU gauge color based on usage.
    pub fn cpu_color(&self) -> ratatui::prelude::Color {
        use ratatui::prelude::Color;
        if self.cpu_percent >= 80.0 {
            Color::Red
        } else if self.cpu_percent >= 60.0 {
            Color::Yellow
        } else {
            Color::Green
        }
    }

    /// Get RAM gauge color based on usage.
    pub fn ram_color(&self) -> ratatui::prelude::Color {
        use ratatui::prelude::Color;
        let ratio = self.ram_ratio();
        if ratio >= 0.8 {
            Color::Red
        } else if ratio >= 0.6 {
            Color::Yellow
        } else {
            Color::Green
        }
    }
}

/// zkVM task metrics for display.
#[derive(Debug, Clone)]
pub struct ZkVMMetrics {
    /// Total number of tasks executed.
    pub tasks_fetched: usize,
    /// Number of tasks successfully proved.
    pub tasks_submitted: usize,
    /// Total zkVM runtime in seconds.
    pub zkvm_runtime_secs: u64,
    /// Status of the last task.
    pub last_task_status: String,
    /// Total points earned from successful proofs (300 points each).
    pub _total_points: u64,
}

impl Default for ZkVMMetrics {
    fn default() -> Self {
        Self {
            tasks_fetched: 0,
            tasks_submitted: 0,
            zkvm_runtime_secs: 0,
            last_task_status: "None".to_string(),
            _total_points: 0,
        }
    }
}

impl ZkVMMetrics {
    /// Calculate success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        if self.tasks_fetched == 0 {
            0.0
        } else {
            (self.tasks_submitted as f64 / self.tasks_fetched as f64) * 100.0
        }
    }

    /// Format total points with commas for better readability.
    pub fn _format_points(&self) -> String {
        let points = self._total_points;
        if points >= 1_000_000 {
            format!("{:.1}M", points as f64 / 1_000_000.0)
        } else if points >= 1_000 {
            format!("{},{:03}", points / 1_000, points % 1_000)
        } else {
            points.to_string()
        }
    }

    /// Get success rate color based on performance.
    pub fn success_rate_color(&self) -> ratatui::prelude::Color {
        use ratatui::prelude::Color;
        let rate = self.success_rate();
        if rate >= 75.0 {
            Color::Green
        } else if rate >= 50.0 {
            Color::Yellow
        } else {
            Color::Red
        }
    }

    /// Format zkVM runtime as human-readable string.
    pub fn format_runtime(&self) -> String {
        let hours = self.zkvm_runtime_secs / 3600;
        let minutes = (self.zkvm_runtime_secs % 3600) / 60;
        let seconds = self.zkvm_runtime_secs % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}

/// Task fetch state information for accurate timing display.
#[derive(Debug, Clone)]
pub struct TaskFetchInfo {
    /// Current backoff duration in seconds.
    pub backoff_duration_secs: u64,
    /// Time since last fetch attempt in seconds.
    pub time_since_last_fetch_secs: u64,
    /// Whether we can fetch now (no backoff).
    pub can_fetch_now: bool,
}

impl Default for TaskFetchInfo {
    fn default() -> Self {
        Self {
            backoff_duration_secs: 0,
            time_since_last_fetch_secs: 0,
            can_fetch_now: true,
        }
    }
}
