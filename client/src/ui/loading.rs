use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    symbols,
    widgets::{Block, Padding, Paragraph, Widget, canvas::Canvas},
};

use crate::{State, ui::center};

pub fn render(area: Rect, buf: &mut Buffer) {
    let message = "loading...";
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
