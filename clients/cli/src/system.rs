//! System information and performance measurements

use cfg_if::cfg_if;
use std::hint::black_box;
use std::process;
use std::sync::OnceLock;
use std::thread::available_parallelism;
use std::time::Instant;
use sysinfo::{CpuRefreshKind, RefreshKind, System};

const NUM_TESTS: u64 = 1_000_000;
const OPERATIONS_PER_ITERATION: u64 = 4; // sin, add, multiply, divide
const NUM_REPEATS: usize = 5; // Number of repeats to average the results

// Cache for flops measurement - only measure once per application run
static FLOPS_CACHE: OnceLock<f32> = OnceLock::new();

/// Get the number of logical cores available on the machine.
pub fn num_cores() -> usize {
    available_parallelism().map(|n| n.get()).unwrap_or(1) // Fallback to 1 if detection fails
}

/// Return (logical_cores, base_frequency_MHz).
/// `sysinfo` provides MHz on every supported OS.
fn cpu_stats() -> (u64, u64) {
    let mut sys =
        System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()));
    // Wait a bit because CPU usage is based on diff.
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    // Refresh CPUs again to get actual value.
    sys.refresh_cpu_all();

    let logical_cores = available_parallelism().map(|n| n.get() as u64).unwrap_or(1);

    // `sysinfo` reports the *base* frequency of the first CPU package.
    // This avoids transient turbo clocks that overestimate peak GFLOP/s.
    let base_mhz = match sys.cpus().first() {
        Some(cpu) => cpu.frequency(),
        None => 0, // Fallback if no CPUs are detected
    };

    (logical_cores, base_mhz)
}

/// Detect the number of double-precision floating-point operations
/// a single **core** can theoretically complete per clock cycle,
/// based on the best SIMD extension available on *this* build target
/// (not at run-time).
fn flops_per_cycle_per_core() -> u32 {
    cfg_if! {
        if #[cfg(all(target_arch = "x86_64", target_feature = "avx512f"))] {
            // 512-bit vectors → 16 FP64 ops per FMA instruction
            16
        } else if #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))] {
            // 256-bit vectors → 8 FP64 ops
            8
        } else if #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))] {
            // 128-bit vectors → 4 FP64 ops
            4
        } else {
            // Conservative scalar fallback
            1
        }
    }
}

/// Estimate peak FLOPS (in GFLOP/s) from the number of prover threads and clock speed.
pub fn estimate_peak_gflops(num_provers: usize) -> f64 {
    let (_cores, mhz) = cpu_stats();
    let fpc = flops_per_cycle_per_core() as u64;

    // GFLOP/s = (cores * MHz * flops_per_cycle) / 1000
    (num_provers as u64 * mhz * fpc) as f64 / 1000.0
}

/// Measure actual FLOPS (in GFLOP/s) of this machine by running mathematical operations.
/// The result is cached after the first measurement, so subsequent calls return the cached value.
pub fn measure_gflops() -> f32 {
    *FLOPS_CACHE.get_or_init(|| {
        let num_cores: u64 = match available_parallelism() {
            Ok(cores) => cores.get() as u64,
            Err(_) => {
                eprintln!(
                    "Warning: Unable to determine the number of logical cores. Defaulting to 1."
                );
                1
            }
        };

        let avg_flops: f64 = (0..NUM_REPEATS)
            .map(|_| {
                let start = Instant::now();

                let total_flops: u64 = (0..num_cores)
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
    })
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

// We encode the memory usage to i32 type at client
fn bytes_to_mb_i32(bytes: u64) -> i32 {
    // Convert to MB with 3 decimal places of precision
    // Multiply by 1000 to preserve 3 decimal places
    ((bytes as f64 * 1000.0) / 1_048_576.0).round() as i32
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_estimate_peak_gflops() {
        let num_provers = 4; // Example number of prover threads
        let gflops = super::estimate_peak_gflops(num_provers);
        // println!("gflops = {}", gflops);
        assert!(gflops > 0.0, "Expected positive GFLOP/s estimate");
    }

    #[test]
    fn test_cpu_stats() {
        let (cores, mhz) = super::cpu_stats();
        assert!(cores > 0, "Expected at least one core");
        assert!(mhz > 0, "Expected non-zero MHz");
        // println!("Cores: {}, Base Frequency: {} MHz", cores, mhz);
    }
}
