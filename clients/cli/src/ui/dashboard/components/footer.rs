//! Dashboard footer component
//!
//! Renders footer with quit instructions and version info

use super::super::state::DashboardState;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

/// Render enhanced footer.
pub fn render_footer(f: &mut Frame, area: ratatui::layout::Rect, state: &DashboardState) {
    let footer_text = if state.update_available {
        if let Some(latest) = &state.latest_version {
            format!(
                "[Q] Quit | NEW VERSION {} AVAILABLE! Visit github.com/nexus-xyz/nexus-cli",
                latest
            )
        } else {
            "[Q] Quit | New version available! Visit github.com/nexus-xyz/nexus-cli".to_string()
        }
    } else {
        "[Q] Quit | Nexus Prover Dashboard".to_string()
    };

    let footer_color = if state.update_available {
        Color::LightYellow
    } else {
        Color::Cyan
    };

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
