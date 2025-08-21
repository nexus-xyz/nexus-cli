//! Dashboard metrics components
//!
//! Renders system and zkVM metrics

use super::super::state::DashboardState;
use super::super::utils::format_compact_timestamp;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Gauge, Padding, Paragraph, Wrap};

/// Render enhanced metrics section with better layout.
pub fn render_metrics_section(f: &mut Frame, area: ratatui::layout::Rect, state: &DashboardState) {
    let metrics_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_system_metrics(f, metrics_chunks[0], state);
    render_zkvm_metrics(f, metrics_chunks[1], state);
}

/// Render enhanced system metrics with better gauges.
pub fn render_system_metrics(f: &mut Frame, area: ratatui::layout::Rect, state: &DashboardState) {
    let metrics = &state.system_metrics;

    // Responsive gauge layout - each gauge gets equal space
    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33), // CPU gauge
            Constraint::Percentage(33), // RAM gauge
            Constraint::Percentage(34), // Peak RAM (slightly larger for rounding)
        ])
        .split(area);

    // CPU gauge with enhanced styling
    let cpu_gauge = Gauge::default()
        .block(
            Block::default()
                .title("CPU Usage")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(metrics.cpu_color())),
        )
        .gauge_style(
            Style::default()
                .fg(metrics.cpu_color())
                .add_modifier(Modifier::BOLD),
        )
        .percent((metrics.cpu_percent as u16).min(100))
        .label(format!("{:.1}%", metrics.cpu_percent));

    // RAM gauge with enhanced styling
    let ram_gauge = Gauge::default()
        .block(
            Block::default()
                .title("RAM Usage")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(metrics.ram_color())),
        )
        .gauge_style(
            Style::default()
                .fg(metrics.ram_color())
                .add_modifier(Modifier::BOLD),
        )
        .percent((metrics.ram_ratio() * 100.0) as u16)
        .label(format!(
            "{} / {:.1}GB",
            metrics.format_ram(),
            state.total_ram_gb
        ));

    // Peak RAM gauge
    let peak_gauge = Gauge::default()
        .block(
            Block::default()
                .title("Peak RAM")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::LightBlue)),
        )
        .gauge_style(
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
        )
        .percent((metrics.peak_ram_ratio() * 100.0) as u16)
        .label(metrics.format_peak_ram());

    f.render_widget(cpu_gauge, gauge_chunks[0]);
    f.render_widget(ram_gauge, gauge_chunks[1]);
    f.render_widget(peak_gauge, gauge_chunks[2]);
}

/// Render enhanced zkVM metrics panel.
pub fn render_zkvm_metrics(f: &mut Frame, area: ratatui::layout::Rect, state: &DashboardState) {
    let metrics = &state.zkvm_metrics;
    let mut zkvm_lines = Vec::new();

    // // Points display - most prominent metric
    // TODO: Add points display back in when we have a way to get pointsq
    // zkvm_lines.push(Line::from(vec![
    //     Span::styled("Points: ", Style::default().fg(Color::Gray)),
    //     Span::styled(
    //         metrics.format_points(),
    //         Style::default()
    //             .fg(Color::LightYellow)
    //             .add_modifier(Modifier::BOLD),
    //     ),
    // ]));

    // TODO: Add zkVM KHz display here, once we have a way to measure it locally.

    // Tasks statistics
    zkvm_lines.push(Line::from(vec![
        Span::styled("Tasks: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", metrics.tasks_fetched),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    zkvm_lines.push(Line::from(vec![
        Span::styled("Completed: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{} / {}", metrics.tasks_submitted, metrics.tasks_fetched),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // Success rate with color coding
    let success_text = format!("{:.1}%", metrics.success_rate());
    zkvm_lines.push(Line::from(vec![
        Span::styled("Success: ", Style::default().fg(Color::Gray)),
        Span::styled(
            success_text,
            Style::default()
                .fg(metrics.success_rate_color())
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // Runtime information
    zkvm_lines.push(Line::from(vec![
        Span::styled("Runtime: ", Style::default().fg(Color::Gray)),
        Span::styled(metrics.format_runtime(), Style::default().fg(Color::Cyan)),
    ]));

    // Last task info
    let status_color = match metrics.last_task_status.as_str() {
        "Success" => Color::Green,
        "Failed" => Color::Red,
        _ => Color::Gray,
    };
    zkvm_lines.push(Line::from(vec![
        Span::styled("Last: ", Style::default().fg(Color::Gray)),
        Span::styled(&metrics.last_task_status, Style::default().fg(status_color)),
    ]));

    // Show timestamp of last successful submission instead of duration
    let last_submission_text = if let Some(timestamp) = state.last_submission_timestamp() {
        format_compact_timestamp(timestamp)
    } else {
        "Never".to_string()
    };
    zkvm_lines.push(Line::from(vec![
        Span::styled("Last Proof: ", Style::default().fg(Color::Gray)),
        Span::styled(last_submission_text, Style::default().fg(Color::Yellow)),
    ]));

    let zkvm_block = Block::default()
        .title("zkVM STATS")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::uniform(1));

    let zkvm_paragraph = Paragraph::new(zkvm_lines)
        .block(zkvm_block)
        .wrap(Wrap { trim: true });
    f.render_widget(zkvm_paragraph, area);
}
