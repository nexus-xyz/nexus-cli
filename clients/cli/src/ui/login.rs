//! Login screen module

use ratatui::Frame;
use ratatui::prelude::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

/// Renders the login screen with a simple message and instructions.
pub fn render_login(f: &mut Frame) {
    let size = f.area();

    let block = Block::default()
        .title("Login")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new("Press Enter to login\nPress Esc to exit").block(block);

    f.render_widget(paragraph, size);
}
