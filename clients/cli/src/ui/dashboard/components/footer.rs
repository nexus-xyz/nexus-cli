//! Dashboard footer component
//!
//! Renders footer with quit instructions and version info

use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

/// Render enhanced footer.
pub fn render_footer(f: &mut Frame, area: ratatui::layout::Rect) {
    let footer_text = "[Q] Quit | Nexus Prover Dashboard".to_string();

    let footer_color = Color::Cyan;

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(footer_color)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_type(BorderType::Thick),
        );
    f.render_widget(footer, area);
}
