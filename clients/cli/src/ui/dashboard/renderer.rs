//! Dashboard main renderer

use super::components::{footer, header, info_panel, logs, metrics};
use super::state::DashboardState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Color, Style};
use ratatui::widgets::Block;

pub fn render_dashboard(f: &mut Frame, state: &DashboardState) {
    if state.with_background_color {
        f.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 20, 24))),
            f.area(),
        );
    }

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Fill(1),
            Constraint::Percentage(35),
            Constraint::Length(2),
        ])
        .margin(1)
        .split(f.area());

    header::render_header(f, main_chunks[0], state);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(main_chunks[1]);

    info_panel::render_info_panel(f, content_chunks[0], state);
    logs::render_logs_panel(f, content_chunks[1], state);
    metrics::render_metrics_section(f, main_chunks[2], state);
    footer::render_footer(f, main_chunks[3]);
}
