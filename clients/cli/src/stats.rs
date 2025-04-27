use crate::flops;
use crate::memory_stats;
use std::time::{Duration, SystemTime};

pub struct Stats {
    pub flops: f32,
    pub program_memory_mb: f32,
    pub total_memory_mb: f32,
    pub start_time: SystemTime,

    pub memory_utilization: f32,
    pub time_online: Duration,
    pub proofs_completed: u32,
    pub proofs_per_hour: f32,
}

impl Stats {
    pub fn new() -> Self {
        let (program_memory, total_memory) = memory_stats::get_memory_info();
        let program_memory_mb = memory_stats::mb_i32_to_f32(program_memory);
        let total_memory_mb = memory_stats::mb_i32_to_f32(total_memory);

        Stats {
            flops: 0.0,
            program_memory_mb,
            total_memory_mb,
            start_time: SystemTime::now(),
            memory_utilization: program_memory_mb / total_memory_mb,
            time_online: Duration::from_secs(0),
            proofs_completed: 0,
            proofs_per_hour: 0.0,
        }
    }

    pub fn update(&mut self) {
        self.flops = flops::measure_flops();

        let (program_memory, total_memory) = memory_stats::get_memory_info();
        self.program_memory_mb = memory_stats::mb_i32_to_f32(program_memory);
        self.total_memory_mb = memory_stats::mb_i32_to_f32(total_memory);
        self.memory_utilization = self.program_memory_mb / self.total_memory_mb;

        self.time_online = SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or(Duration::from_secs(0));

        let hours_online = self.time_online.as_secs_f32() / 3600.0;
        if hours_online > 0.0 {
            self.proofs_per_hour = self.proofs_completed as f32 / hours_online;
        }
    }

    pub fn increment_proof_count(&mut self) {
        self.proofs_completed += 1;
        self.update();
    }
}
