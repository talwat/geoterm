use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Color,
    symbols,
    widgets::{
        Block, Widget,
        canvas::{self, Canvas, Circle, Map},
    },
};
use shared::lobby::Clients;

pub struct Results {
    pub data: shared::RoundData,
    pub lobby: Clients,
}

fn convert_color(c: shared::Color) -> Color {
    match c {
        shared::Color::Red => Color::Red,
        shared::Color::Yellow => Color::Yellow,
        shared::Color::Green => Color::Green,
        shared::Color::Blue => Color::Blue,
        shared::Color::Magenta => Color::Magenta,
    }
}

impl Widget for &Results {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(format!(" Round {} ", self.data.number))
            .title_alignment(Alignment::Center);

        Canvas::default()
            .marker(symbols::Marker::HalfBlock)
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
            .paint(|ctx| {
                ctx.draw(&Map {
                    resolution: canvas::MapResolution::High,
                    color: Color::White,
                });

                for player in &self.data.players {
                    let options = &self
                        .lobby
                        .into_iter()
                        .find(|x| x.id == player.id)
                        .unwrap()
                        .options;
                    let guess = player.guess.unwrap();

                    ctx.draw(&Circle {
                        x: guess.longitude as f64,
                        y: guess.latitude as f64,
                        radius: 4.0,
                        color: convert_color(options.color),
                    });
                }
            })
            .block(block)
            .render(area, buf);
    }
}
