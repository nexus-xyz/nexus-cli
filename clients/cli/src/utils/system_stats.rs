//! System stats

use rayon::prelude::*;
use std::hint::black_box;
use std::process;
use std::thread::available_parallelism;
use std::time::Instant;
use sysinfo::System;

/// Estimate FLOPS (in GFLOP/s) of this machine.
pub fn measure_gflops() -> f32 {
    const NUM_TESTS: u64 = 1_000_000;
    const OPERATIONS_PER_ITERATION: u64 = 4; // sin, add, multiply, divide
    const NUM_REPEATS: usize = 5; // Number of repeats to average the results

    let num_cores: u64 = match available_parallelism() {
        Ok(cores) => cores.get() as u64,
        Err(_) => {
            eprintln!("Warning: Unable to determine the number of logical cores. Defaulting to 1.");
            1
        }
    };
    println!("Using {} logical cores for FLOPS measurement", num_cores);

    let avg_flops: f64 = (0..NUM_REPEATS)
        .map(|_| {
            let start = Instant::now();

            let total_flops: u64 = (0..num_cores)
                .into_par_iter()
                .map(|_| {
                    let mut x: f64 = 1.0;
                    for _ in 0..NUM_TESTS {
                        x = black_box((x.sin() + 1.0) * 0.5 / 1.1);
                    }
                    NUM_TESTS * OPERATIONS_PER_ITERATION
                })
                .sum();

            total_flops as f64 / start.elapsed().as_secs_f64()
        })
        .sum::<f64>()
        / NUM_REPEATS as f64; // Average the FLOPS over all repeats

    (avg_flops / 1e9) as f32
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
