//! Splash screen rendering module.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub const LOGO_NAME: &str = r#"
  ███╗   ██╗  ███████╗  ██╗  ██╗  ██╗   ██╗  ███████╗
  ████╗  ██║  ██╔════╝  ╚██╗██╔╝  ██║   ██║  ██╔════╝
  ██╔██╗ ██║  █████╗     ╚███╔╝   ██║   ██║  ███████╗
  ██║╚██╗██║  ██╔══╝     ██╔██╗   ██║   ██║  ╚════██║
  ██║ ╚████║  ███████╗  ██╔╝ ██╗  ╚██████╔╝  ███████║
  ╚═╝  ╚═══╝  ╚══════╝  ╚═╝  ╚═╝   ╚═════╝   ╚══════╝
"#;

pub fn render_splash(f: &mut Frame) {
    // Convert LOGO_NAME into styled Lines
    let mut lines: Vec<Line> = LOGO_NAME
        .trim_matches('\n')
        .lines()
        .map(|line| {
            Span::styled(
                line.to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .into()
        })
        .collect();

    // Add a spacer line
    lines.push(Line::from(Span::raw(" ")));

    // Add version line
    lines.push(
        Span::styled(
            format!("Version {}", env!("CARGO_PKG_VERSION")),
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::ITALIC),
        )
        .into(),
    );

    // Determine the logo height
    let logo_height = (lines.len() + 2) as u16;

    // Vertically center using layout
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min((f.area().height.saturating_sub(logo_height)) / 2),
            Constraint::Length(logo_height),
            Constraint::Min((f.area().height.saturating_sub(logo_height + 1)) / 2),
        ])
        .split(f.area());

    let centered_area: Rect = vertical_chunks[1];

    // Create the centered paragraph
    let logo = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(logo, centered_area);
}
