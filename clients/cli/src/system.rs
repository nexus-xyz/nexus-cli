//! System information and performance measurements

use std::process;
use std::thread::available_parallelism;
use sysinfo::System;

/// Get the number of logical cores available on the machine.
pub fn num_cores() -> usize {
    available_parallelism().map(|n| n.get()).unwrap_or(1) // Fallback to 1 if detection fails
}

/// Total memory in GB of the machine.
pub fn total_memory_gb() -> f64 {
    let mut sys = System::new();
    sys.refresh_memory();
    let total_memory = sys.total_memory(); // bytes
    total_memory as f64 / 1000.0 / 1000.0 / 1000.0 // Convert to GB
}

/// Memory used by the current process, in GB.
#[allow(unused)]
pub fn process_memory_gb() -> f64 {
    let mut sys = System::new();
    sys.refresh_all();

    let current_pid = process::id();
    let current_process = sys
        .process(sysinfo::Pid::from(current_pid as usize))
        .expect("Failed to get current process");

    let memory = current_process.memory(); // bytes
    memory as f64 / 1000.0 / 1000.0 / 1000.0 // Convert to GB
}

/// Estimate peak FLOPS (in GFLOP/s) from the number of prover threads and clock speed.
pub fn estimate_peak_gflops(num_provers: usize) -> f32 {
    // Assuming 4 operations per cycle
    let peak_flops = (num_provers as f32) * 4.0 * 2.0e9; // Assumes 2 GHz clock speed
    peak_flops / 1e9 // Convert to GFLOP/s
}

/// Get the memory usage of the current process and the total system memory, in MB.
pub fn get_memory_info() -> (i32, i32) {
    let mut system = System::new_all();
    system.refresh_all();

    let current_pid = process::id();
    let current_process = system
        .process(sysinfo::Pid::from(current_pid as usize))
        .expect("Failed to get current process");

    let program_memory_mb = bytes_to_mb_i32(current_process.memory());
    let total_memory_mb = bytes_to_mb_i32(system.total_memory());

    (program_memory_mb, total_memory_mb)
}

// We encode the memory usage to i32 type at client
fn bytes_to_mb_i32(bytes: u64) -> i32 {
    // Convert to MB with 3 decimal places of precision
    // Multiply by 1000 to preserve 3 decimal places
    ((bytes as f64 * 1000.0) / 1_048_576.0).round() as i32
}
