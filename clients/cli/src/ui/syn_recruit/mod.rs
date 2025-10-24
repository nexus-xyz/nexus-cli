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
        
        // Initialize with the first INFO log entry
        let mut activity_logs = Vec::new();
        activity_logs.push("[INFO] In A.D. 2,0,2,5, SYN was beginning.".to_string());
        
        Self {
            current_scene: 0,
            start_time: Instant::now(),
            last_update: Instant::now(),
            system_metrics: SystemMetrics::default(),
            sysinfo,
            current_line: String::new(),
            full_line: String::new(),
            current_speaker: String::new(),
            activity_logs,
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
                        // Skip 0x0000 scene entirely since it's shown as INFO log
                        if speaker == "0x0000" {
                            self.current_scene += 1;
                            return;
                        }
                        
                        self.current_speaker = speaker.to_string();
                        self.full_line = line.to_string();
                        self.current_line.clear();
                        self.char_index = 0;
                        self.typing_state = TypingState::Typing;
                        self.last_char_time = Instant::now();
                        
                        // Add speaker to activity log
                        self.activity_logs.push(format!("[{}] {}", speaker, ""));
                        // Play gentle tap sound for new log entry
                        self.play_tap_sound();
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
                        
                        // Keep only the last 20 log entries to prevent overflow
                        if self.activity_logs.len() > 20 {
                            self.activity_logs.remove(0);
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
            (Duration::from_millis(0), "0x0000", "In A.D. 2,0,2,5, SYN was beginning."),
            (Duration::from_millis(1200), "0x0001", "What happen?"),
            (Duration::from_millis(1900), "0x0002", "Somebody set up us the cron."),
            (Duration::from_millis(2700), "0x0003", "We get signal."),
            (Duration::from_millis(3300), "0x0001", "What!"),
            (Duration::from_millis(3700), "0x0003", "Main screen turn on."),
            (Duration::from_millis(4000), "0xACCC", "How are you sysadmins!!"),
            (Duration::from_millis(4400), "0x0001", "It's you!!"),
            (Duration::from_millis(5000), "0xACCC", "All your node are belong to us."),
            (Duration::from_millis(5800), "0xACCC", "You are on the way to destruction."),
            (Duration::from_millis(6700), "0x0001", "What you say!!"),
            (Duration::from_millis(7400), "0xACCC", "You have no chance to survive make your time."),
            (Duration::from_millis(8300), "0xACCC", "Ha ha ha ha...."),
            (Duration::from_millis(10600), "0x0001", "Take off every 'SYNC'!!"),
            (Duration::from_millis(11500), "0x0001", "You know what you doing."),
            (Duration::from_millis(12200), "0x0001", "Move 'SYNC'."),
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
                self.activity_logs.push("✗ [ERR] Rate limited by server: zkVM task submission failed".to_string());
                // Keep only the last 20 log entries
                if self.activity_logs.len() > 20 {
                    self.activity_logs.remove(0);
                }
            }
        } else if self.current_scene >= 7 && self.current_scene < 9 {
            // 0xACCC villain appears - system alert
            self.cpu_spike = 85.0;
            if self.activity_logs.len() < 6 {
                self.activity_logs.push("[ALERT] Unauthorized access detected from 0xACCC".to_string());
                // Keep only the last 20 log entries
                if self.activity_logs.len() > 20 {
                    self.activity_logs.remove(0);
                }
            }
        } else if self.current_scene >= 14 && self.current_scene < 16 {
            // "Take off every 'SYNC'" - SYN flood begins
            self.cpu_spike = 95.0;
            if self.activity_logs.len() < 7 {
                self.activity_logs.push("[INFO] SYN flood protocols initiated".to_string());
                // Keep only the last 20 log entries
                if self.activity_logs.len() > 20 {
                    self.activity_logs.remove(0);
                }
            }
        } else if self.current_scene >= 16 && self.current_scene < 18 {
            // "Move 'SYNC'" - rocket launch
            self.cpu_spike = 90.0;
            if self.activity_logs.len() < 8 {
                self.activity_logs.push("[OK] SYN packets launched successfully".to_string());
                // Keep only the last 20 log entries
                if self.activity_logs.len() > 20 {
                    self.activity_logs.remove(0);
                }
            }
        } else if self.current_scene >= 18 {
            // Victory - system normalizes
            self.cpu_spike = 25.0;
            if self.activity_logs.len() < 9 {
                self.activity_logs.push("[OK] FOR GREAT JUSTICE - Mission complete".to_string());
                // Keep only the last 20 log entries
                if self.activity_logs.len() > 20 {
                    self.activity_logs.remove(0);
                }
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

    fn play_tap_sound(&self) {
        // Pleasant tap sound for new log entries
        // Use a soft, musical combination for a gentle notification
        print!("\x08\x08\x08"); // Triple backspace for a soft, pleasant tap
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

    fn get_success_rate(&self) -> f64 {
        // Success rate logic based on story progression
        if self.current_scene <= 0 {
            // Intro - high success rate
            100.0
        } else if self.current_scene >= 1 && self.current_scene <= 15 {
            // During crisis - success rate drops to 0%
            0.0
        } else if self.current_scene >= 16 {
            // Move SYNC and after - success rate restored
            100.0
        } else {
            50.0 // Default
        }
    }

    fn get_task_count(&self) -> u32 {
        // Dynamic task counting based on story progression with fast updates
        let elapsed_milliseconds = self.start_time.elapsed().as_millis();
        
        if self.current_scene <= 0 {
            // Intro - tasks climbing rapidly (10+ updates per second)
            let base_tasks = 1000000;
            let additional_tasks = (elapsed_milliseconds * 50) as u32; // Fast updates
            (base_tasks + additional_tasks).min(23953940)
        } else if self.current_scene >= 1 && self.current_scene <= 15 {
            // During ACCC crisis - tasks stop climbing
            let base_tasks = 1000000 + (elapsed_milliseconds * 50) as u32;
            // Stop at the peak when ACCC shows up (around scene 7)
            if self.current_scene >= 7 {
                23953940 // Peak reached, no more growth
            } else {
                base_tasks.min(23953940)
            }
        } else if self.current_scene >= 16 {
            // Move SYNC - tasks resume climbing rapidly
            let base_tasks = 23953940; // Start from peak
            let additional_tasks = ((elapsed_milliseconds - 10000) * 30).max(0) as u32;
            base_tasks + additional_tasks
        } else {
            1000000 // Default
        }
    }

    fn get_rocket_positions(&self) -> Vec<usize> {
        // Get animated rocket positions based on tick
        let base_positions = vec![0, 5, 10, 15, 20];
        base_positions.iter().map(|&pos| (pos + self.tick) % 25).collect()
    }

    fn get_fade_intensity(&self) -> f32 {
        // Calculate fade intensity based on completion time
        if !self.is_complete {
            return 0.0;
        }
        let completion_time = self.start_time.elapsed().as_secs_f32();
        let pause_start = 13.0; // Pause for 2 seconds after completion
        let fade_start = 15.0; // Start fading after pause
        if completion_time < pause_start {
            0.0 // Pause - no fade
        } else {
            ((completion_time - fade_start) / 3.0).clamp(0.0, 1.0) // Fade over 3 seconds
        }
    }

    fn should_show_progressive_fade(&self) -> bool {
        // Show progressive fade after completion
        self.is_complete
    }

    fn get_progressive_fade_progress(&self) -> f32 {
        if !self.should_show_progressive_fade() {
            return 0.0;
        }
        
        let completion_time = self.start_time.elapsed().as_secs_f32();
        let fade_start = 13.0; // Start fading after completion
        let fade_duration = 4.0; // Fade over 4 seconds
        
        ((completion_time - fade_start) / fade_duration).clamp(0.0, 1.0)
    }
    fn should_show_rocket_fill(&self) -> bool {
        // Show rocket fill during "You know what you doing" (scene 15) to "Move SYNC" (scene 16)
        self.current_scene >= 15 && self.current_scene <= 16
    }

    fn get_rocket_fill_progress(&self) -> f32 {
        if !self.should_show_rocket_fill() {
            return 0.0;
        }
        
        // Calculate progress based on scene and time within scene
        let scene_progress = if self.current_scene == 15 {
            // "You know what you doing" - start filling
            0.0
        } else if self.current_scene == 16 {
            // "Move SYNC" - complete filling
            1.0
        } else {
            0.0
        };
        
        // Add time-based progression within the current scene
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let scene_start_time = if self.current_scene == 15 { 11.5 } else { 12.2 };
        let time_in_scene = (elapsed - scene_start_time).max(0.0);
        
        // Gradual fill over 2 seconds total
        let time_progress = (time_in_scene / 2.0).clamp(0.0, 1.0);
        
        // Combine scene progress with time progress
        (scene_progress + time_progress * 0.5).clamp(0.0, 1.0)
    }
}

pub fn render_syn_recruit(f: &mut Frame, state: &SynRecruitState) {
    // Check if we should show progressive fade
    let fade_progress = state.get_progressive_fade_progress();
    
    if fade_progress > 0.0 {
        // Progressive fade to rockets with SYN
        render_progressive_fade(f, state, fade_progress);
    } else {
        // Normal rendering
        render_normal_ui(f, state);
    }
}

fn render_progressive_fade(f: &mut Frame, state: &SynRecruitState, fade_progress: f32) {
    // Create animated rockets moving to the right
    let base_rocket_line = "🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀";
    
    // Calculate offset based on tick for movement
    let offset = (state.tick / 2) % 3; // Move every 2 ticks, cycle every 3 positions
    let rocket_line = match offset {
        0 => base_rocket_line,
        1 => " 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀",
        2 => "  🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀",
        _ => base_rocket_line,
    };
    
    // Create full screen of rockets
    let mut lines = vec![];
    for _ in 0..f.area().height {
        lines.push(Line::from(rocket_line));
    }
    
    // Add SYN display in center
    let syn_text = vec![
        Line::from(""),
        Line::from(""),
        Line::from(""),
        Line::from(""),
        Line::from(""),
        Line::from(" ███████╗██╗   ██╗███╗   ██╗"),
        Line::from(" ██╔════╝╚██╗ ██╔╝████╗  ██║"),
        Line::from(" ███████╗ ╚████╔╝ ██╔██╗ ██║"),
        Line::from(" ╚════██║  ╚██╔╝  ██║╚██╗██║"),
        Line::from(" ███████║   ██║   ██║ ╚████║"),
        Line::from(" ╚══════╝   ╚═╝   ╚═╝  ╚═══╝"),
        Line::from(""),
        Line::from(""),
        Line::from(" FOR GREAT JUSTICE"),
        Line::from(""),
        Line::from(""),
    ];
    
    // Replace center lines with SYN display
    let start_line = (f.area().height as usize - syn_text.len()) / 2;
    for (i, syn_line) in syn_text.iter().enumerate() {
        if start_line + i < lines.len() {
            lines[start_line + i] = syn_line.clone();
        }
    }
    
    let syn_display = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
    
    f.render_widget(syn_display, f.area());
}

fn render_normal_ui(f: &mut Frame, state: &SynRecruitState) {
    // Use the same background as the real CLI
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(16, 20, 24))),
        f.area(),
    );
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header (reduced from 4 to 3)
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

    // Info panel (left side) - always active
    render_info_panel(f, content_chunks[0], state);
    
    // Activity log (center) - always active
    render_activity_log(f, content_chunks[1], state);
    
    // Main screen (right side) - shows empty until main screen turns on
    render_main_screen(f, content_chunks[2], state);
    
    // Metrics section (bottom) - always visible
    render_metrics_section(f, main_chunks[2], state);
    
    // Footer - always visible
    render_footer(f, main_chunks[3], state);
}

fn render_header(f: &mut Frame, area: ratatui::layout::Rect, state: &SynRecruitState) {
    let header_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(1)]) // Progress bar now 1 height
        .split(area);

    // Title section - mimicking Nexus CLI
    let title_text = "SYN CREW INTERFACE v0.10.17";
    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM).border_type(BorderType::Thick));
    f.render_widget(title, header_chunks[0]);

    // Progress gauge showing team activity level
    let team_activity_percent = state.get_team_activity_percent();
    let progress_text = format!("SYN Online: {:.0}%", team_activity_percent);
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

fn render_info_panel(f: &mut Frame, area: ratatui::layout::Rect, _state: &SynRecruitState) {
    let info_text = vec![
        Line::from(Span::styled("Status: Online", Style::default().fg(Color::Green))),
        Line::from(Span::styled("Env: Production", Style::default().fg(Color::Green))),
        Line::from(Span::styled("Version: v0.10.17", Style::default().fg(Color::Cyan))),
        Line::from(Span::styled("Threads: 1753600", Style::default().fg(Color::LightYellow))),
        Line::from(Span::styled("Memory: 2304 TB", Style::default().fg(Color::LightCyan))),
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
    let content = if state.current_scene < 5 {
        // Show empty panel until "main screen turn on" (scene 5)
        vec![Line::from("")]
    } else if state.is_complete {
        // Mission complete - show progressive fade to rockets with SYN
        let fade_intensity = state.get_fade_intensity();
        let mut lines = vec![];
        
        // Fill entire screen with rockets
        let rocket_line = "🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀";
        
        // Create full screen of rockets
        for _ in 0..20 {
            lines.push(Line::from(rocket_line));
        }
        
        lines
    } else if state.should_show_rocket_fill() {
        // During "Take off every SYNC" to "Move SYNC" - gradually fill screen with rockets
        let fill_progress = state.get_rocket_fill_progress();
        let mut lines = vec![];
        
        // Calculate how many lines to fill with rockets - use full area height
        let total_lines = area.height as usize - 2; // Account for padding
        let filled_lines = (total_lines as f32 * fill_progress) as usize;
        
        // Add rocket-filled lines
        let rocket_line = "🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀";
        for _ in 0..filled_lines {
            lines.push(Line::from(rocket_line));
        }
        
        // Add empty lines for remaining space
        for _ in filled_lines..total_lines {
            lines.push(Line::from(""));
        }
        
        lines
    } else if state.current_speaker == "0xACCC" {
        // Show triangle made of dots for ACCC
        vec![
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from("           •"),
            Line::from("          • •"),
            Line::from("         • • •"),
            Line::from("        • • • •"),
            Line::from("       • • • • •"),
            Line::from("      • • • • • •"),
            Line::from("     • • • • • • •"),
            Line::from("    • • • • • • • •"),
            Line::from("   • • • • • • • • •"),
            Line::from("  • • • • • • • • • •"),
            Line::from(""),
            Line::from(""),
            Line::from(""),
        ]
    } else {
        // Show empty for all other speakers (CABB, F1X3, D00D, etc.)
        vec![Line::from("")]
    };

    let color = if state.current_speaker == "0xACCC" {
        Color::Magenta
    } else {
        Color::Green
    };

    // Apply fade to black effect if needed
    let fade_intensity = state.get_fade_intensity();
    let bg_color = if fade_intensity > 0.0 {
        // Fade to black based on intensity
        let fade_amount = (fade_intensity * 255.0) as u8;
        Color::Rgb(fade_amount, fade_amount, fade_amount)
    } else {
        Color::Rgb(16, 20, 24) // Normal background
    };

    let main_screen = Paragraph::new(Text::from(content))
        .block(Block::default()
            .title("MAIN SCREEN")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(color))
            .padding(Padding::uniform(1))
            .style(Style::default().bg(bg_color)))
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
            } else if log.starts_with("[0x0001]") {
                Color::Yellow
            } else if log.starts_with("[0x0002]") {
                Color::Green
            } else if log.starts_with("[0x0003]") {
                Color::Cyan
            } else if log.starts_with("[0x0000]") {
                Color::Gray
            } else if log.contains("[ERR]") {
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
            
            // Add info icon for INFO entries
            let display_text = if log.contains("[INFO]") {
                format!("{}", log)
            } else {
                log.clone()
            };
            
            ListItem::new(Span::styled(display_text, Style::default().fg(color)))
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
    let success_rate = state.get_success_rate();
    let zkvm_lines = vec![
        Line::from(vec![
            Span::styled("Tasks: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", task_count), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Success: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1}%", success_rate),
                Style::default().fg(if success_rate >= 80.0 { Color::Green } else { Color::Red }).add_modifier(Modifier::BOLD)
            ),
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
        "[Q] No quitting - 🚀"
    } else {
        "[Q] No quitting"
    };
    
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, area);
}
