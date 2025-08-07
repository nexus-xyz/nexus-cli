//! Dashboard info panel component
//!
//! Renders system information panel

use crate::environment::Environment;

use super::super::state::DashboardState;
use ratatui::Frame;
use ratatui::prelude::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap};

/// Render enhanced info panel with better styling.
pub fn render_info_panel(f: &mut Frame, area: ratatui::layout::Rect, state: &DashboardState) {
    let mut info_lines = Vec::new();

    // Node information with enhanced formatting
    let node_text = if let Some(id) = state.node_id {
        format!("Node: {}", id)
    } else {
        "Node: Disconnected".to_string()
    };
    info_lines.push(Line::from(vec![Span::styled(
        node_text,
        Style::default().fg(Color::LightBlue),
    )]));

    // Environment with color coding
    let env_color = match state.environment {
        Environment::Production => Color::Green,
        Environment::Custom {
            orchestrator_url: _,
        } => Color::Yellow,
    };
    info_lines.push(Line::from(vec![Span::styled(
        format!("Env: {}", state.environment),
        Style::default().fg(env_color),
    )]));

    // Version info
    let version = env!("CARGO_PKG_VERSION");
    info_lines.push(Line::from(vec![Span::styled(
        format!("Version: {}", version),
        Style::default().fg(Color::Cyan),
    )]));

    // Uptime with better formatting
    let uptime = state.start_time.elapsed();
    let uptime_string = if uptime.as_secs() >= 86400 {
        format!(
            "Uptime: {}d {}h {}m",
            uptime.as_secs() / 86400,
            (uptime.as_secs() % 86400) / 3600,
            (uptime.as_secs() % 3600) / 60
        )
    } else if uptime.as_secs() >= 3600 {
        format!(
            "Uptime: {}h {}m {}s",
            uptime.as_secs() / 3600,
            (uptime.as_secs() % 3600) / 60,
            uptime.as_secs() % 60
        )
    } else {
        format!(
            "Uptime: {}m {}s",
            uptime.as_secs() / 60,
            uptime.as_secs() % 60
        )
    };
    info_lines.push(Line::from(vec![Span::styled(
        uptime_string,
        Style::default().fg(Color::LightGreen),
    )]));

    // Threads info
    info_lines.push(Line::from(vec![Span::styled(
        format!("Threads: {}", state.num_threads),
        Style::default().fg(Color::LightYellow),
    )]));

    // Total memory
    info_lines.push(Line::from(vec![Span::styled(
        format!("Memory: {:.1} GB", state.total_ram_gb),
        Style::default().fg(Color::LightCyan),
    )]));

    // Note: Task ID removed from system info as requested

    let info_block = Block::default()
        .title("SYSTEM INFO")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::uniform(1));

    let info_paragraph = Paragraph::new(info_lines)
        .block(info_block)
        .wrap(Wrap { trim: true });
    f.render_widget(info_paragraph, area);
}
