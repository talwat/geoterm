use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    widgets::{Block, Padding, Paragraph, Widget},
};

use crate::ui::center;

pub fn render(area: Rect, buf: &mut Buffer, message: &str) {
    let centered = center(
        area,
        Constraint::Length(message.len() as u16 + 4),
        Constraint::Length(3),
    );

    Paragraph::new(message)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::bordered().padding(Padding::horizontal(1)))
        .render(centered, buf);
}
