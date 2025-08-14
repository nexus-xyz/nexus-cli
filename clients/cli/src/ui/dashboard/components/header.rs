//! Dashboard header component
//!
//! Renders the title and progress gauge

use super::super::state::DashboardState;
use crate::events::ProverState;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Gauge, Paragraph};

/// Render enhanced header with title and stage progress.
pub fn render_header(f: &mut Frame, area: ratatui::layout::Rect, state: &DashboardState) {
    let header_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(2)])
        .split(area);

    // Title section with enhanced version display
    let version = env!("CARGO_PKG_VERSION");
    let title_text = if state.update_available {
        if let Some(latest) = &state.latest_version {
            format!("NEXUS PROVER v{} -> {} UPDATE AVAILABLE", version, latest)
        } else {
            format!("NEXUS PROVER v{} - UPDATE AVAILABLE", version)
        }
    } else {
        format!("NEXUS PROVER v{}", version)
    };

    let title_color = if state.update_available {
        Color::LightYellow
    } else {
        Color::Cyan
    };

    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_type(BorderType::Thick),
        );
    f.render_widget(title, header_chunks[0]);

    // Gauge logic: proving takes priority, then task fetching countdown
    let (progress_text, gauge_color, progress_percent) = {
        // Check if we're currently proving
        match state.current_prover_state() {
            ProverState::Proving => {
                // Animated proving gauge - loops every 20 ticks for smooth animation
                let progress = ((state.tick % 20) as f64 / 20.0 * 100.0) as u16;
                (
                    "PROVING - Generating proof".to_string(),
                    Color::LightGreen,
                    progress,
                )
            }
            ProverState::Waiting => {
                // Task fetching countdown logic
                let fetch_info = &state.task_fetch_info;
                if !fetch_info.can_fetch_now && fetch_info.backoff_duration_secs > 0 {
                    let remaining_secs = fetch_info
                        .backoff_duration_secs
                        .saturating_sub(fetch_info.time_since_last_fetch_secs);
                    let progress = if fetch_info.backoff_duration_secs > 0 {
                        ((fetch_info.time_since_last_fetch_secs as f64
                            / fetch_info.backoff_duration_secs as f64)
                            * 100.0) as u16
                    } else {
                        100
                    };
                    let display_text = if remaining_secs > 0 {
                        format!("WAITING - Ready for next task ({}s)", remaining_secs)
                    } else {
                        "WAITING - Ready for next task".to_string()
                    };
                    (display_text, Color::LightBlue, progress.min(100))
                } else {
                    (
                        "WAITING - Ready for next task".to_string(),
                        Color::LightBlue,
                        100,
                    )
                }
            }
        }
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .gauge_style(
            Style::default()
                .fg(gauge_color)
                .add_modifier(Modifier::BOLD),
        )
        .percent(progress_percent)
        .label(progress_text);

    f.render_widget(gauge, header_chunks[1]);
}
