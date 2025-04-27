use std::process;
use sysinfo::System;

// We encode the memory usage to i32 type at client
pub fn bytes_to_mb_i32(bytes: u64) -> i32 {
    // Convert to MB with 3 decimal places of precision
    // Multiply by 1000 to preserve 3 decimal places
    ((bytes as f64 * 1000.0) / 1_048_576.0).round() as i32
}

// At server, we decode the memory usage from i32 to f32 to get correct memory usage
pub fn mb_i32_to_f32(mb: i32) -> f32 {
    // Convert back to f32, dividing by 1000 to get the correct value
    (mb as f32) / 1000.0
}

pub fn get_memory_info() -> (i32, i32) {
    let mut system = System::new_all();
    system.refresh_all();

    let current_pid = process::id();
    let current_process = system
        .process(sysinfo::Pid::from(current_pid as usize))
        .expect("Failed to get current process");

    let program_memory = current_process.memory();
    let total_memory = system.total_memory();

    (
        bytes_to_mb_i32(program_memory),
        bytes_to_mb_i32(total_memory),
    )
}

pub fn calculate_memory_utilization() -> f32 {
    let (program_memory, total_memory) = get_memory_info();
    let program_mb = mb_i32_to_f32(program_memory);
    let total_mb = mb_i32_to_f32(total_memory);
    program_mb / total_mb
}

pub fn format_memory_usage() -> String {
    let (program_memory, total_memory) = get_memory_info();
    let program_mb = mb_i32_to_f32(program_memory);
    let total_mb = mb_i32_to_f32(total_memory);
    let utilization = program_mb / total_mb * 100.0;
    
    format!("{:.2} MB / {:.2} MB ({:.1}%)", program_mb, total_mb, utilization)
}
