//! SYN Recruitment Video TUI Module
//!
//! This module provides a TUI-based recruitment video experience that integrates
//! with the existing dashboard system, showing real-time system metrics during
//! the "All Your Node Are Belong To Us" parody.

use crate::ui::metrics::SystemMetrics;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::text::{Line, Text};
use std::time::{Duration, Instant};
use sysinfo::System;

/// State for the SYN recruitment video
#[derive(Debug)]
pub struct SynRecruitState {
    /// Current scene index
    pub current_scene: usize,
    /// Start time of the video
    pub start_time: Instant,
    /// Last update time
    pub last_update: Instant,
    /// System metrics for real-time monitoring
    pub system_metrics: SystemMetrics,
    /// System info instance for CPU monitoring
    pub sysinfo: System,
    /// Current dialogue line
    pub current_line: String,
    /// Current speaker
    pub current_speaker: String,
    /// Activity log entries
    pub activity_logs: Vec<String>,
    /// Whether the video is complete
    pub is_complete: bool,
    /// Current CPU spike level (0-100)
    pub cpu_spike: f32,
    /// Current memory usage spike
    pub memory_spike: f32,
    /// Animation tick counter
    pub tick: usize,
}

impl SynRecruitState {
    pub fn new() -> Self {
        let mut sysinfo = System::new_all();
        sysinfo.refresh_all();
        
        Self {
            current_scene: 0,
            start_time: Instant::now(),
            last_update: Instant::now(),
            system_metrics: SystemMetrics::default(),
            sysinfo,
            current_line: String::new(),
            current_speaker: String::new(),
            activity_logs: Vec::new(),
            is_complete: false,
            cpu_spike: 0.0,
            memory_spike: 0.0,
            tick: 0,
        }
    }

    pub fn update(&mut self) {
        self.tick += 1;
        self.last_update = Instant::now();
        
        // Update system metrics
        self.sysinfo.refresh_all();
        self.system_metrics.cpu_percent = self.sysinfo.global_cpu_usage();
        self.system_metrics.ram_bytes = self.sysinfo.used_memory();
        self.system_metrics.total_ram_bytes = self.sysinfo.total_memory();
        
        // Update the recruitment video based on elapsed time
        self.update_scene();
        
        // Simulate CPU spikes based on story events
        self.update_cpu_spikes();
    }

    fn update_scene(&mut self) {
        let elapsed = self.start_time.elapsed();
        
        // Define scene timing based on the original "All your base" structure
        let scenes = [
            (Duration::from_millis(0), "0xDEAD", "In A.D. 20,1,5, SYN was beginning."),
            (Duration::from_millis(1200), "0xCABB", "What happen?"),
            (Duration::from_millis(1900), "0xF1X3", "Somebody set up us the cron."),
            (Duration::from_millis(2700), "0xD00D", "We get signal."),
            (Duration::from_millis(3300), "0xCABB", "What!"),
            (Duration::from_millis(3700), "0xD00D", "Main screen turn on."),
            (Duration::from_millis(4400), "0xCABB", "It's you!!"),
            (Duration::from_millis(5000), "0xACCC", "How are you sysadmins!!"),
            (Duration::from_millis(5800), "0xACCC", "All your node are belong to us."),
            (Duration::from_millis(6700), "0xACCC", "You are on the way to destruction."),
            (Duration::from_millis(7600), "0xCABB", "What you say!!"),
            (Duration::from_millis(8300), "0xACCC", "You have no chance to survive — make your time."),
            (Duration::from_millis(9200), "0xACCC", "Ha ha ha ha...."),
            (Duration::from_millis(10100), "0xD00D", "0xCABB!!"),
            (Duration::from_millis(10600), "0xCABB", "Take off every 'SYNC'!!"),
            (Duration::from_millis(11500), "0xCABB", "You know what you doing."),
            (Duration::from_millis(12200), "0xCABB", "Move 'SYNC'."),
            (Duration::from_millis(12900), "0xCABB", "For great justice."),
        ];

        // Find current scene based on elapsed time
        let mut current_scene_idx = 0;
        for (i, (time, speaker, line)) in scenes.iter().enumerate() {
            if elapsed >= *time {
                current_scene_idx = i;
                self.current_speaker = speaker.to_string();
                self.current_line = line.to_string();
            } else {
                break;
            }
        }

        self.current_scene = current_scene_idx;
        
        // Check if video is complete
        if elapsed >= Duration::from_millis(13900) {
            self.is_complete = true;
        }
    }

    fn update_cpu_spikes(&mut self) {
        let elapsed = self.start_time.elapsed();
        
        // CPU spikes based on story events
        if elapsed >= Duration::from_millis(1900) && elapsed < Duration::from_millis(2700) {
            // "Somebody set up us the cron" - CPU spike to 100%
            self.cpu_spike = 100.0;
            if self.activity_logs.len() < 5 {
                self.activity_logs.push("[CRIT] Cron job detected: CPU usage at 100%".to_string());
            }
        } else if elapsed >= Duration::from_millis(5000) && elapsed < Duration::from_millis(5800) {
            // ACK villain appears - system alert
            self.cpu_spike = 85.0;
            if self.activity_logs.len() < 6 {
                self.activity_logs.push("[ALERT] Unauthorized access detected from 0xACCC".to_string());
            }
        } else if elapsed >= Duration::from_millis(10600) && elapsed < Duration::from_millis(11500) {
            // "Take off every 'SYNC'" - SYN flood begins
            self.cpu_spike = 95.0;
            if self.activity_logs.len() < 7 {
                self.activity_logs.push("[INFO] SYN flood protocols initiated".to_string());
            }
        } else if elapsed >= Duration::from_millis(12200) && elapsed < Duration::from_millis(12900) {
            // "Move 'SYNC'" - rocket launch
            self.cpu_spike = 90.0;
            if self.activity_logs.len() < 8 {
                self.activity_logs.push("[OK] SYN packets launched successfully".to_string());
            }
        } else if elapsed >= Duration::from_millis(12900) {
            // Victory - system normalizes
            self.cpu_spike = 25.0;
            if self.activity_logs.len() < 9 {
                self.activity_logs.push("[OK] FOR GREAT JUSTICE - Mission complete".to_string());
            }
        } else {
            // Normal operation
            self.cpu_spike = self.system_metrics.cpu_percent;
        }

        // Memory spike simulation
        if self.cpu_spike > 80.0 {
            self.memory_spike = 85.0;
        } else {
            self.memory_spike = (self.system_metrics.ram_bytes as f32 / self.system_metrics.total_ram_bytes as f32) * 100.0;
        }
    }

    fn get_speaker_color(&self, speaker: &str) -> Color {
        match speaker {
            "0xACCC" => Color::Magenta,
            "0xCABB" => Color::Yellow,
            "0xF1X3" => Color::Green,
            "0xD00D" => Color::Cyan,
            "0xDEAD" => Color::Gray,
            _ => Color::White,
        }
    }
}

pub fn render_syn_recruit(f: &mut Frame, state: &SynRecruitState) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Current dialogue
            Constraint::Fill(1),   // Main content
            Constraint::Length(8), // Activity logs
            Constraint::Length(3), // System metrics
        ])
        .margin(1)
        .split(f.area());

    // Header
    let header = Paragraph::new("🎬 SYN RECRUITMENT VIDEO - All Your Node Are Belong To Us")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Current dialogue
    let dialogue_color = state.get_speaker_color(&state.current_speaker);
    let dialogue_text = if !state.current_line.is_empty() {
        format!("{}: {}", state.current_speaker, state.current_line)
    } else {
        "Initializing SYN system...".to_string()
    };
    
    let dialogue = Paragraph::new(dialogue_text)
        .style(Style::default().fg(dialogue_color))
        .block(Block::default().borders(Borders::ALL).title("Dialogue"));
    f.render_widget(dialogue, chunks[1]);

    // Main content area - show ASCII art or system status
    let main_content = if state.is_complete {
        // Show SYN ASCII art
        let ascii_art = vec![
            Line::from("██████╗ ██╗   ██╗ ███╗   ██╗"),
            Line::from("██╔══██╗╚██╗ ██╔╝ ████╗  ██║"),
            Line::from("██████╔╝ ╚████╔╝  ██╔██╗ ██║"),
            Line::from("██╔══██╗  ╚██╔╝   ██║╚██╗██║"),
            Line::from("██████╔╝   ██║    ██║ ╚████║"),
            Line::from("╚═════╝    ╚═╝    ╚═╝  ╚═══╝"),
            Line::from(""),
            Line::from("                 A.D. 20,1,5"),
        ];
        Paragraph::new(Text::from(ascii_art))
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title("SYN"))
    } else {
        // Show system status with emojis
        let status_text = if state.cpu_spike > 90.0 {
            "🚀🚀🚀 SYN PACKETS LAUNCHING... 🚀🚀🚀"
        } else if state.cpu_spike > 80.0 {
            "⚠️  SYSTEM ALERT - ACK FORCES DETECTED ⚠️"
        } else if state.current_speaker == "0xCABB" && state.current_line.contains("Move") {
            "🚀 MOVE 'SYNC'! 🚀"
        } else if state.current_speaker == "0xCABB" && state.current_line.contains("justice") {
            "🤖🦾 FOR GREAT JUSTICE! 🤖🦾"
        } else {
            "🔄 SYN SYSTEM OPERATIONAL 🔄"
        };
        
        Paragraph::new(status_text)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Status"))
    };
    f.render_widget(main_content, chunks[2]);

    // Activity logs
    let logs_text: Vec<Line> = state.activity_logs
        .iter()
        .map(|log| Line::from(log.as_str()))
        .collect();
    
    let logs = Paragraph::new(Text::from(logs_text))
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("Activity Log"));
    f.render_widget(logs, chunks[3]);

    // System metrics
    let metrics_text = format!(
        "CPU: {:.1}% | Memory: {:.1}% | Scene: {}/18 | Time: {:.1}s",
        state.cpu_spike,
        state.memory_spike,
        state.current_scene + 1,
        state.start_time.elapsed().as_secs_f32()
    );
    
    let metrics = Paragraph::new(metrics_text)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("System Metrics"));
    f.render_widget(metrics, chunks[4]);
}
