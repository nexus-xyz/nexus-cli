//! Rewards processed notification overlay
//!
//! Displays a congratulatory modal when reportProving returns rewards_processed.
//! Inspired by the "Cache of Points" notification - persists until next proof submission.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

/// Render the rewards_processed congratulations overlay as a centered modal.
/// Call this after rendering the main dashboard when show_rewards_overlay is true.
pub fn render_rewards_overlay(f: &mut Frame, area: Rect) {
    // Modal dimensions - centered, ~60% of area
    let modal_width = (area.width as f32 * 0.6) as u16;
    let modal_height = 14;
    let modal_x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = area.y + (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect {
        x: modal_x,
        y: modal_y,
        width: modal_width,
        height: modal_height,
    };

    // Semi-transparent dark overlay (we use DarkGray as a dimming effect)
    let overlay = Block::default()
        .style(Style::default().bg(Color::Black));
    f.render_widget(overlay, area);

    // Modal block with golden/amber accent
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(255, 193, 7))) // Amber/gold
        .style(Style::default().bg(Color::Rgb(24, 24, 28)));

    let inner = block.inner(modal_area);
    f.render_widget(block, modal_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .margin(1)
        .split(inner);

    // Title: "Congratulations!"
    let title = Paragraph::new("Congratulations!")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Rgb(255, 193, 7))
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title, chunks[0]);

    // Main message: "You've Struck a Cache of Points!"
    let main_msg = Paragraph::new("You've Struck a Cache of Points!")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(main_msg, chunks[1]);

    // Sub message
    let sub_msg = Paragraph::new("Your rewards have been processed. Claim your points!")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true });
    f.render_widget(sub_msg, chunks[2]);

    // Urgency line
    let urgency = Paragraph::new("Notification will dismiss when you submit your next proof.")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray))
        .wrap(Wrap { trim: true });
    f.render_widget(urgency, chunks[3]);
}
