use image::{ImageBuffer, RgbImage};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Color,
    symbols,
    widgets::{
        Block, Padding, Paragraph, Widget,
        canvas::{self, Canvas, Painter},
    },
};

pub struct Round {
    pub images: [RgbImage; 3],
    pub street: String,
    pub number: usize,
    pub guessed: bool,
}

impl Widget for &Round {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let image = &self.images[1];

        let layout = Layout::new(
            ratatui::layout::Direction::Vertical,
            vec![Constraint::Length(2), Constraint::Fill(1)],
        )
        .horizontal_margin(2)
        .vertical_margin(1)
        .split(area);

        Block::bordered()
            .title(format!(" Round {} ", self.number))
            .title_alignment(Alignment::Center)
            .render(area, buf);

        Paragraph::new("(insert timer)").render(layout[0], buf);

        Paragraph::new(self.street.clone())
            .alignment(Alignment::Right)
            .render(layout[0], buf);

        let (width, height) = image.dimensions();
        let (width, height) = (width as f64, height as f64);

        Canvas::default()
            .marker(symbols::Marker::HalfBlock)
            .x_bounds([0.0, width])
            .y_bounds([0.0, height])
            .paint(|ctx| {
                let points = image.enumerate_pixels().map(|(x, y, p)| {
                    let [r, g, b] = p.0;
                    let color = Color::Rgb(r, g, b);
                    let y = height - y as f64;
                    ((x as f64), y, color)
                });

                for (x, y, color) in points {
                    ctx.draw(&canvas::Points {
                        coords: &[(x, y)],
                        color,
                    });
                }
            })
            .render(layout[1], buf);
    }
}
