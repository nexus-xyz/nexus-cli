use colored::Colorize;
use crate::stats::Stats;
use std::time::Duration;

pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let days = total_seconds / (24 * 3600);
    let hours = (total_seconds % (24 * 3600)) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

pub fn create_progress_bar(value: f32, width: usize) -> String {
    let filled_width = (value * width as f32).round() as usize;
    let empty_width = width - filled_width;
    
    let filled = "█".repeat(filled_width);
    let empty = "░".repeat(empty_width);
    
    format!("[{}{}]", filled.green(), empty)
}

pub fn display_stats(stats: &Stats) {
    println!("\n===== {} =====", "Node Statistics".bold().underline().bright_cyan());
    
    println!("{}: {} GFLOPS", 
        "Computational capacity".bold(), 
        format!("{:.2}", stats.flops).bright_cyan());
    
    let memory_bar = create_progress_bar(stats.memory_utilization, 20);
    println!("{}: {} MB / {} MB {} {:.1}%", 
        "Memory usage".bold(),
        format!("{:.2}", stats.program_memory_mb).bright_cyan(),
        format!("{:.2}", stats.total_memory_mb).bright_cyan(),
        memory_bar,
        (stats.memory_utilization * 100.0).bright_cyan());
    
    println!("{}: {}", 
        "Time online".bold(), 
        format_duration(stats.time_online).bright_cyan());
    
    println!("{}: {}", 
        "Proofs completed".bold(), 
        stats.proofs_completed.to_string().bright_cyan());
    
    println!("{}: {}", 
        "Proofs per hour".bold(), 
        format!("{:.2}", stats.proofs_per_hour).bright_cyan());
    
    println!("================================================\n");
}

pub fn display_compact_stats(stats: &Stats) {
    println!("Stats: {} GFLOPS | Mem: {:.1}% | Online: {} | Proofs: {} ({:.2}/hr)",
        format!("{:.2}", stats.flops).bright_cyan(),
        (stats.memory_utilization * 100.0).bright_cyan(),
        format_duration(stats.time_online).bright_cyan(),
        stats.proofs_completed.to_string().bright_cyan(),
        stats.proofs_per_hour.bright_cyan());
}
