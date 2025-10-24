//! SYNC Move Interface TUI Module
//!
//! This module provides a TUI-based SYNC takeover interface that integrates
//! with the existing dashboard system, showing real-time system metrics during
//! the "All Your Node Are Belong To Us" parody - taking off every SYNC.

use crate::ui::metrics::SystemMetrics;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap, Gauge, BorderType, Padding, List, ListItem};
use ratatui::text::{Line, Span, Text};
use ratatui::layout::Alignment;
use ratatui::prelude::Modifier;
use std::time::{Duration, Instant};
use sysinfo::System;
use std::io::Write;

/// State for the SYNC Move interface
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
    /// Current dialogue line being typed
    pub current_line: String,
    /// Full dialogue line to type
    pub full_line: String,
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
    /// Character typing animation state
    pub typing_state: TypingState,
    /// Last character typed time
    pub last_char_time: Instant,
    /// Current character index being typed
    pub char_index: usize,
}

/// State for character-by-character typing animation
#[derive(Debug)]
pub enum TypingState {
    Waiting,        // Waiting for next scene
    Typing,         // Currently typing characters
    Complete,       // Current line complete, waiting for next
    Finished,       // All scenes complete
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
            full_line: String::new(),
            current_speaker: String::new(),
            activity_logs: Vec::new(),
            is_complete: false,
            cpu_spike: 0.0,
            memory_spike: 0.0,
            tick: 0,
            typing_state: TypingState::Waiting,
            last_char_time: Instant::now(),
            char_index: 0,
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
        
        // Handle typing animation
        self.update_typing_animation();
        
        // Update the recruitment video based on elapsed time
        self.update_scene();
        
        // Simulate CPU spikes based on story events
        self.update_cpu_spikes();
    }

    fn update_typing_animation(&mut self) {
        match self.typing_state {
            TypingState::Waiting => {
                // Check if it's time to start typing the next scene
                let elapsed = self.start_time.elapsed();
                let scenes = self.get_scenes();
                
                if self.current_scene < scenes.len() {
                    let (scene_time, speaker, line) = scenes[self.current_scene];
                    if elapsed >= scene_time {
                        self.current_speaker = speaker.to_string();
                        self.full_line = line.to_string();
                        self.current_line.clear();
                        self.char_index = 0;
                        self.typing_state = TypingState::Typing;
                        self.last_char_time = Instant::now();
                        
                        // Add speaker to activity log
                        self.activity_logs.push(format!("[{}] {}", speaker, ""));
                    }
                } else {
                    self.typing_state = TypingState::Finished;
                    self.is_complete = true;
                }
            }
            TypingState::Typing => {
                // Type characters one by one
                if self.char_index < self.full_line.len() {
                    let char_delay = Duration::from_millis(30); // Faster typing speed
                    if self.last_char_time.elapsed() >= char_delay {
                        if let Some(ch) = self.full_line.chars().nth(self.char_index) {
                            self.current_line.push(ch);
                            self.char_index += 1;
                            self.last_char_time = Instant::now();
                            
                            // Play beep sound for each character
                            self.play_beep();
                            
                            // Update the last activity log entry with current text
                            if let Some(last_log) = self.activity_logs.last_mut() {
                                *last_log = format!("[{}] {}", self.current_speaker, self.current_line);
                            }
                        } else {
                            // Character not found, move to next state
                            self.typing_state = TypingState::Complete;
                            self.current_scene += 1;
                        }
                    }
                } else {
                    // Line complete, wait before next scene
                    self.typing_state = TypingState::Complete;
                    self.current_scene += 1;
                }
            }
            TypingState::Complete => {
                // Wait a bit before starting next scene
                let wait_time = Duration::from_millis(1200);
                if self.last_char_time.elapsed() >= wait_time {
                    self.typing_state = TypingState::Waiting;
                }
            }
            TypingState::Finished => {
                self.is_complete = true;
            }
        }
    }

    fn get_scenes(&self) -> Vec<(Duration, &'static str, &'static str)> {
        vec![
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
        ]
    }

    fn update_scene(&mut self) {
        // This method is now handled by update_typing_animation
        // Keep it for CPU spike logic
    }

    fn update_cpu_spikes(&mut self) {
        let _elapsed = self.start_time.elapsed();
        let _scenes = self.get_scenes();
        
        // CPU spikes based on story events
        if self.current_scene >= 1 && self.current_scene < 3 {
            // "What happen?" - CPU spike to 100% with rate limit error
            self.cpu_spike = 100.0;
            if self.activity_logs.len() < 5 {
                self.activity_logs.push("✗ [CRIT] Rate limited by server: zkVM task submission failed".to_string());
            }
        } else if self.current_scene >= 7 && self.current_scene < 9 {
            // ACK villain appears - system alert
            self.cpu_spike = 85.0;
            if self.activity_logs.len() < 6 {
                self.activity_logs.push("[ALERT] Unauthorized access detected from 0xACCC".to_string());
            }
        } else if self.current_scene >= 14 && self.current_scene < 16 {
            // "Take off every 'SYNC'" - SYN flood begins
            self.cpu_spike = 95.0;
            if self.activity_logs.len() < 7 {
                self.activity_logs.push("[INFO] SYN flood protocols initiated".to_string());
            }
        } else if self.current_scene >= 16 && self.current_scene < 18 {
            // "Move 'SYNC'" - rocket launch
            self.cpu_spike = 90.0;
            if self.activity_logs.len() < 8 {
                self.activity_logs.push("[OK] SYN packets launched successfully".to_string());
            }
        } else if self.current_scene >= 18 {
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

    fn play_beep(&self) {
        // Typewriter-like sound (softer click, not warning bell)
        // Use a different bell character for softer sound
        print!("\x08"); // Backspace character for softer click
        std::io::stdout().flush().unwrap_or_default();
    }

    fn get_team_activity_percent(&self) -> f64 {
        // Team activity logic based on story progression
        if self.current_scene <= 0 {
            // Intro - high activity
            90.0
        } else if self.current_scene >= 1 && self.current_scene <= 15 {
            // During the crisis - low activity
            10.0
        } else if self.current_scene >= 16 {
            // "Move 'SYNC'" and after - high activity restored
            90.0
        } else {
            50.0 // Default
        }
    }

    fn get_uptime(&self) -> u64 {
        // Uptime logic: reset to zero during crisis period
        if self.current_scene >= 1 && self.current_scene <= 15 {
            // Crisis period: uptime resets to zero
            0
        } else {
            // Normal operation: show real elapsed time
            self.start_time.elapsed().as_secs()
        }
    }

    fn get_task_count(&self) -> u32 {
        // Dynamic task counting based on story progression
        let elapsed_seconds = self.start_time.elapsed().as_secs();
        
        if self.current_scene <= 0 {
            // Intro - tasks climbing rapidly
            (elapsed_seconds * 50 + 1000).min(5000) as u32
        } else if self.current_scene >= 1 && self.current_scene <= 15 {
            // During ACCC crisis - tasks stop climbing
            let base_tasks = (self.start_time.elapsed().as_secs() * 50 + 1000).min(5000) as u32;
            // Stop at the peak when ACCC shows up (around scene 7)
            if self.current_scene >= 7 {
                5000 // Peak reached, no more growth
            } else {
                base_tasks
            }
        } else if self.current_scene >= 16 {
            // Move SYNC - tasks resume climbing
            let base_tasks = 5000; // Start from peak
            let additional_tasks = ((self.start_time.elapsed().as_secs() - 10) * 30).max(0) as u32;
            base_tasks + additional_tasks
        } else {
            1000 // Default
        }
    }
}

pub fn render_syn_recruit(f: &mut Frame, state: &SynRecruitState) {
    // Use the same background as the real CLI
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(16, 20, 24))),
        f.area(),
    );

    // Create layout matching the real Nexus CLI
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // Header
            Constraint::Fill(1),    // Main content
            Constraint::Percentage(35), // Metrics
            Constraint::Length(2),  // Footer
        ])
        .margin(1)
        .split(f.area());

    // Render header (mimicking Nexus CLI)
    render_header(f, main_chunks[0], state);

    // Main content area - split like real CLI
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(60), Constraint::Percentage(20)])
        .split(main_chunks[1]);

    // Info panel (left side)
    render_info_panel(f, content_chunks[0], state);
    
    // Activity log (center) - main script dialogue with scrolling
    render_activity_log(f, content_chunks[1], state);
    
    // Main screen (right side) - shows team logos
    render_main_screen(f, content_chunks[2], state);
    
    // Metrics section (bottom)
    render_metrics_section(f, main_chunks[2], state);
    
    // Footer
    render_footer(f, main_chunks[3], state);
}

fn render_header(f: &mut Frame, area: ratatui::layout::Rect, state: &SynRecruitState) {
    let header_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(2)])
        .split(area);

    // Title section - mimicking Nexus CLI
    let title_text = "SYN NODE INTERFACE v0.10.17";
    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM).border_type(BorderType::Thick));
    f.render_widget(title, header_chunks[0]);

    // Progress gauge showing team activity level
    let team_activity_percent = state.get_team_activity_percent();
    let progress_text = format!("Team Activity: {:.0}%", team_activity_percent);
    let gauge_color = if team_activity_percent >= 80.0 {
        Color::Green
    } else if team_activity_percent >= 50.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(gauge_color))
        .percent(team_activity_percent as u16)
        .label(progress_text);
    f.render_widget(gauge, header_chunks[1]);
}

fn render_info_panel(f: &mut Frame, area: ratatui::layout::Rect, state: &SynRecruitState) {
    let info_text = vec![
        Line::from(Span::styled("Status: Online", Style::default().fg(Color::Green))),
        Line::from(Span::styled("Env: Production", Style::default().fg(Color::Green))),
        Line::from(Span::styled("Version: v0.10.17", Style::default().fg(Color::Cyan))),
        Line::from(Span::styled(
            format!("Uptime: {}s", state.get_uptime()),
            Style::default().fg(Color::LightGreen)
        )),
        Line::from(Span::styled("Threads: 1", Style::default().fg(Color::LightYellow))),
        Line::from(Span::styled("Memory: 8.0 GB", Style::default().fg(Color::LightCyan))),
    ];

    let info_panel = Paragraph::new(Text::from(info_text))
        .block(Block::default()
            .title("SYSTEM INFO")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .padding(Padding::uniform(1)))
        .wrap(Wrap { trim: true });
    f.render_widget(info_panel, area);
}

fn render_main_screen(f: &mut Frame, area: ratatui::layout::Rect, state: &SynRecruitState) {
    let content = if state.current_speaker == "0xACCC" {
        // Show ACK team logo when ACCC is speaking
        vec![
            Line::from(""),
            Line::from(""),
            Line::from("        ╭─────────╮"),
            Line::from("       ╱           ╲"),
            Line::from("      ╱             ╲"),
            Line::from("     ╱       ●       ╲"),
            Line::from("    ╱                 ╲"),
            Line::from("   ╱         ▼         ╲"),
            Line::from("  ╱                     ╲"),
            Line::from(" ╱                       ╲"),
            Line::from("╱                         ╲"),
            Line::from("╲                         ╱"),
            Line::from(" ╲                       ╱"),
            Line::from("  ╲                     ╱"),
            Line::from("   ╲                   ╱"),
            Line::from("    ╲                 ╱"),
            Line::from("     ╲               ╱"),
            Line::from("      ╲             ╱"),
            Line::from("       ╲           ╱"),
            Line::from("        ╰─────────╯"),
            Line::from(""),
            Line::from("        TEAM ACK"),
            Line::from(""),
            Line::from("    All your node are"),
            Line::from("      belong to us"),
            Line::from(""),
        ]
    } else {
        // Default system status
        vec![
            Line::from(""),
            Line::from(""),
            Line::from("    ███████╗██╗   ██╗███╗   ██╗"),
            Line::from("    ██╔════╝╚██╗ ██╔╝████╗  ██║"),
            Line::from("    ███████╗ ╚████╔╝ ██╔██╗ ██║"),
            Line::from("    ╚════██║  ╚██╔╝  ██║╚██╗██║"),
            Line::from("    ███████║   ██║   ██║ ╚████║"),
            Line::from("    ╚══════╝   ╚═╝   ╚═╝  ╚═══╝"),
            Line::from(""),
            Line::from("        SYN NODE"),
            Line::from(""),
            Line::from("    System Online"),
            Line::from("    Team Active"),
            Line::from(""),
            Line::from(""),
        ]
    };

    let color = if state.current_speaker == "0xACCC" {
        Color::Magenta
    } else {
        Color::Green
    };

    let main_screen = Paragraph::new(Text::from(content))
        .block(Block::default()
            .title("MAIN SCREEN")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(color))
            .padding(Padding::uniform(1)))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(main_screen, area);
}

fn render_activity_log(f: &mut Frame, area: ratatui::layout::Rect, state: &SynRecruitState) {
    // Create scrollable list items with proper color coding
    let list_items: Vec<ListItem> = state.activity_logs
        .iter()
        .map(|log| {
            // Color code based on speaker and log type
            let color = if log.starts_with("[0xACCC]") {
                Color::Magenta
            } else if log.starts_with("[0xCABB]") {
                Color::Yellow
            } else if log.starts_with("[0xF1X3]") {
                Color::Green
            } else if log.starts_with("[0xD00D]") {
                Color::Cyan
            } else if log.starts_with("[0xDEAD]") {
                Color::Gray
            } else if log.contains("[CRIT]") {
                Color::Red
            } else if log.contains("[ALERT]") {
                Color::Yellow
            } else if log.contains("[INFO]") {
                Color::Cyan
            } else if log.contains("[OK]") {
                Color::Green
            } else {
                Color::White
            };
            ListItem::new(Span::styled(log.as_str(), Style::default().fg(color)))
        })
        .collect();
    
    // Create a scrollable list widget with proper scrolling
    let logs = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("Activity Log"));
    f.render_widget(logs, area);
}

fn render_metrics_section(f: &mut Frame, area: ratatui::layout::Rect, state: &SynRecruitState) {
    let metrics_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // System metrics (left side) - matching real CLI
    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // CPU gauge
            Constraint::Percentage(50), // RAM gauge
        ])
        .split(metrics_chunks[0]);

    // CPU gauge with enhanced styling
    let team_activity = state.get_team_activity_percent();
    let cpu_color = if team_activity >= 80.0 { Color::Green } else { Color::Red };
    let cpu_gauge = Gauge::default()
        .block(
            Block::default()
                .title("CPU Usage")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(cpu_color)),
        )
        .gauge_style(
            Style::default()
                .fg(cpu_color)
                .add_modifier(Modifier::BOLD),
        )
        .percent((state.cpu_spike as u16).min(100))
        .label(format!("{:.1}%", state.cpu_spike));

    // RAM gauge with enhanced styling
    let ram_color = if team_activity >= 80.0 { Color::Green } else { Color::Red };
    let ram_gauge = Gauge::default()
        .block(
            Block::default()
                .title("RAM Usage")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(ram_color)),
        )
        .gauge_style(
            Style::default()
                .fg(ram_color)
                .add_modifier(Modifier::BOLD),
        )
        .percent((state.memory_spike as u16).min(100))
        .label(format!("{:.1}%", state.memory_spike));

    f.render_widget(cpu_gauge, gauge_chunks[0]);
    f.render_widget(ram_gauge, gauge_chunks[1]);

    // zkVM stats (right side) - matching real CLI
    let task_count = state.get_task_count();
    let zkvm_lines = vec![
        Line::from(vec![
            Span::styled("Tasks: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", task_count), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Completed: ", Style::default().fg(Color::Gray)),
            Span::styled(
                if team_activity >= 80.0 { "1 / 1" } else { "0 / 1" },
                Style::default().fg(if team_activity >= 80.0 { Color::Green } else { Color::Red }).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::styled("Success: ", Style::default().fg(Color::Gray)),
            Span::styled(
                if team_activity >= 80.0 { "100.0%" } else { "0.0%" },
                Style::default().fg(if team_activity >= 80.0 { Color::Green } else { Color::Red }).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::styled("Runtime: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.1}s", state.start_time.elapsed().as_secs_f32()), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Last: ", Style::default().fg(Color::Gray)),
            Span::styled("Pending", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Last Proof: ", Style::default().fg(Color::Gray)),
            Span::styled("Never", Style::default().fg(Color::Yellow)),
        ]),
    ];

    let zkvm_block = Block::default()
        .title("zkVM STATS")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::uniform(1));

    let zkvm_paragraph = Paragraph::new(zkvm_lines)
        .block(zkvm_block)
        .wrap(Wrap { trim: true });
    f.render_widget(zkvm_paragraph, metrics_chunks[1]);
}

fn render_footer(f: &mut Frame, area: ratatui::layout::Rect, state: &SynRecruitState) {
    let footer_text = if state.is_complete {
        "🚀 SYN NODE COMPLETE - All Your Node Are Belong To Us! Press any key to exit."
    } else {
        "🚀 SYN NODE INTERFACE - Press any key to exit"
    };
    
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, area);
}
