//! Dashboard logs panel component
//!
//! Renders activity logs with event formatting

use super::super::state::DashboardState;
use super::super::utils::{clean_http_error_message, format_compact_timestamp, get_worker_color};
use crate::events::EventType;
use crate::logging::LogLevel;
use ratatui::Frame;
use ratatui::prelude::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap};

/// Render enhanced logs panel with better event formatting.
pub fn render_logs_panel(f: &mut Frame, area: ratatui::layout::Rect, state: &DashboardState) {
    // Calculate how many log lines can fit in the available area
    // Account for borders and padding (subtract 3 for top/bottom borders + padding)
    let max_logs = (area.height.saturating_sub(3)) as usize;
    let log_count = if max_logs > 0 { max_logs } else { 1 };

    let log_lines: Vec<Line> = state
        .activity_logs
        .iter()
        .filter(|event| event.should_display())
        .rev()
        .take(log_count) // Show as many logs as fit in terminal
        .map(|event| {
            let status_icon = match (event.event_type, event.log_level) {
                (EventType::Success, _) => "✅",
                (EventType::Error, LogLevel::Error) => "❌",
                (EventType::Error, LogLevel::Warn) => "",
                (EventType::Error, _) => "❌",
                (EventType::Refresh, _) => "",
                (EventType::Waiting, _) => "",
                (EventType::StateChange, _) => "", // StateChange events shouldn't be displayed, but add for completeness
            };

            let worker_color = get_worker_color(&event.worker);
            let compact_time = format_compact_timestamp(&event.timestamp);
            let cleaned_msg = clean_http_error_message(&event.msg);

            // Don't truncate - let ratatui handle wrapping naturally
            Line::from(vec![
                Span::raw(format!("{} ", status_icon)),
                Span::styled(
                    format!("{} ", compact_time),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(cleaned_msg, Style::default().fg(worker_color)),
            ])
        })
        .collect();

    let log_paragraph = if log_lines.is_empty() {
        Paragraph::new(vec![Line::from("Starting up...")])
    } else {
        Paragraph::new(log_lines)
    };

    let logs_block = Block::default()
        .title("ACTIVITY LOG")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::uniform(1));

    let log_widget = log_paragraph.block(logs_block).wrap(Wrap { trim: true });

    f.render_widget(log_widget, area);
}
