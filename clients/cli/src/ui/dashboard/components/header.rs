//! Dashboard header component
//!
//! Renders the title and progress gauge

use super::super::state::DashboardState;
use super::super::utils::get_current_state_elapsed_secs;
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

    // Enhanced stage progress using state events with timing
    let elapsed_secs = get_current_state_elapsed_secs(&state.events, state.current_prover_state());
    let (progress_text, gauge_color, progress_percent) = match state.current_prover_state() {
        ProverState::Fetching => (
            format!(
                "🔍 FETCHING - Requesting task from orchestrator ({}s)",
                elapsed_secs
            ),
            Color::Cyan,
            25,
        ),
        ProverState::Proving => (
            format!(
                "⚡ PROVING - Computing zero-knowledge proof ({}s)",
                elapsed_secs
            ),
            Color::Yellow,
            50,
        ),
        ProverState::Submitting => (
            format!(
                "📤 SUBMITTING - Sending proof to network ({}s)",
                elapsed_secs
            ),
            Color::Green,
            75,
        ),
        ProverState::Waiting => (
            format!("⏳ WAITING - Ready for next task ({}s)", elapsed_secs),
            Color::LightBlue,
            100,
        ),
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
