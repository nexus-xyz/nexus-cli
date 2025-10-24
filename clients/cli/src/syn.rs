use serde::Deserialize;
use std::fs;
use std::thread;
use std::time::Duration;
use crate::audio::{AudioEngine, generate_background_music, generate_sound_effects};

#[derive(Deserialize)]
struct Scene {
    speaker: String,
    line: String,
    delay_ms: u64,
}

#[derive(Deserialize)]
struct LogEntry {
    level: String,
    msg: String,
}

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GRAY: &str = "\x1b[90m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";

fn speaker_color(speaker: &str) -> &str {
    match speaker {
        "0xACCC" => MAGENTA,
        "0xCABB" => YELLOW,
        "0xF1X3" => GREEN,
        "0xD00D" => CYAN,
        "0xDEAD" => GRAY,
        _ => RESET,
    }
}

fn level_color(level: &str) -> &str {
    match level {
        "INFO" => CYAN,
        "WARN" => YELLOW,
        "ALERT" => MAGENTA,
        "CRIT" => RED,
        "ERROR" => RED,
        "OK" => GREEN,
        _ => GRAY,
    }
}

fn now() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

async fn print_activity(logs: &[LogEntry]) {
    println!("{DIM}── Activity Log ───────────────────────────────{RESET}");
    for entry in logs {
        println!(
            "[{GRAY}{}{RESET}] {}{}{RESET} {}",
            now(),
            level_color(&entry.level),
            entry.level,
            entry.msg
        );
        thread::sleep(Duration::from_millis(200));
    }
    println!("{DIM}───────────────────────────────────────────────{RESET}");
}

async fn rocket_launch() {
    println!("{YELLOW}MOVE 'SYNC'! 🚀🚀🚀{RESET}");
    thread::sleep(Duration::from_millis(800));
}

async fn robot_arms_celebration() {
    println!("{GREEN}FOR GREAT JUSTICE! 🤖🦾{RESET}");
    thread::sleep(Duration::from_millis(1000));
}

async fn print_ascii(file_path: &str, color: &str) {
    if let Ok(content) = fs::read_to_string(file_path) {
        for line in content.lines() {
            println!("{}{}{}", color, line, RESET);
            thread::sleep(Duration::from_millis(15));
        }
    }
}

pub async fn run_syn_recruit() -> Result<(), Box<dyn std::error::Error>> {
    // Generate audio files if they don't exist
    if !std::path::Path::new("../../assets/audio").exists() {
        generate_sound_effects()?;
        generate_background_music()?;
    }
    
    // Initialize audio engine
    let audio_engine = AudioEngine::new()?;
    
    // Clear screen
    print!("\x1b[2J\x1b[H");
    
    // Read data files - adjust paths to be relative to project root
    let scenes_content = fs::read_to_string("../../scripts/syn/all-your-node.scenes.json")?;
    let logs_content = fs::read_to_string("../../scripts/syn/activity.log.json")?;
    
    let scenes: Vec<Scene> = serde_json::from_str(&scenes_content)?;
    let logs: Vec<LogEntry> = serde_json::from_str(&logs_content)?;
    
    println!("{GRAY}BOOT> INITIALIZING SYN SYSTEM ...{RESET}");
    thread::sleep(Duration::from_millis(400));
    
    // Start background music
    let _ = audio_engine.play_sound("../../assets/audio/syn_bg_music.wav");
    
    for scene in scenes {
        // Play console beep for each message
        let _ = audio_engine.play_sound("../../assets/audio/console_beep.wav");
        
        println!(
            "{}{}{}{}: {}",
            BOLD,
            speaker_color(&scene.speaker),
            scene.speaker,
            RESET,
            scene.line
        );
        thread::sleep(Duration::from_millis(scene.delay_ms));
        
        if scene.line.contains("Take off every 'SYNC'") {
            // Play alert sound
            let _ = audio_engine.play_sound("../../assets/audio/alert.wav");
            print_activity(&logs[0..5]).await;
        }
        
        if scene.line.contains("Move 'SYNC'") {
            // Play rocket sound
            let _ = audio_engine.play_sound("../../assets/audio/rocket.wav");
            rocket_launch().await;
            print_activity(&logs[5..8]).await;
        }
        
        if scene.line.contains("For great justice") {
            // Play victory sound
            let _ = audio_engine.play_sound("../../assets/audio/victory.wav");
            robot_arms_celebration().await;
            print_activity(&logs[8..]).await;
        }
    }
    
    print_ascii("../../assets/ascii/outro-syn.txt", GREEN).await;
    println!("{DIM}Broadcast complete. Packet dropped.{RESET}");
    
    // Stop audio
    audio_engine.stop();
    
    Ok(())
}
