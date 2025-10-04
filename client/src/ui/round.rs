use image::RgbImage;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Color,
    symbols,
    widgets::{
        Block, Paragraph, Widget,
        canvas::{self, Canvas, Context, Map},
    },
};

pub struct Round {
    pub images: [RgbImage; 3],
    pub street: String,
    pub number: usize,
    pub guessed: bool,
    pub guessing: bool,
    pub cursor: (f32, f32),
}

impl Round {
    fn draw_guesser(&self, ctx: &mut Context<'_>) {
        ctx.draw(&Map {
            resolution: canvas::MapResolution::High,
            color: Color::White,
        });

        ctx.draw(&canvas::Points {
            coords: &[(self.cursor.0 as f64, self.cursor.1 as f64)],
            color: Color::Red,
        });
    }

    fn draw_image(&self, ctx: &mut Context<'_>, height: f64) {
        let points = self.images[1].enumerate_pixels().map(|(x, y, p)| {
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
    }
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
            .x_bounds(if self.guessing {
                [-180.0, 180.0]
            } else {
                [0.0, width]
            })
            .y_bounds(if self.guessing {
                [-90.0, 90.0]
            } else {
                [0.0, height]
            })
            .paint(|ctx| {
                if self.guessing {
                    self.draw_guesser(ctx);
                } else {
                    self.draw_image(ctx, height);
                }
            })
            .render(layout[1], buf);
    }
}
