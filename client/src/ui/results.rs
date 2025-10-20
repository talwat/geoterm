use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{
        Block, Padding, Paragraph, Widget,
        canvas::{self, Canvas, Map, Points},
    },
};
use shared::lobby::Clients;

pub struct Results {
    pub id: usize,
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
        let areas: [Rect; 2] = Layout::new(
            ratatui::layout::Direction::Vertical,
            [Constraint::Fill(1), Constraint::Percentage(25)],
        )
        .areas(area);

        Canvas::default()
            .marker(symbols::Marker::HalfBlock)
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
            .paint(|ctx| {
                ctx.draw(&Map {
                    resolution: canvas::MapResolution::High,
                    color: Color::DarkGray,
                });

                for player in &self.data.players {
                    let options = &self
                        .lobby
                        .into_iter()
                        .find(|x| x.id == player.id)
                        .unwrap()
                        .options;
                    let guess = player.guess.unwrap();

                    ctx.draw(&Points {
                        coords: &[(guess.longitude as f64, guess.latitude as f64)],
                        color: convert_color(options.color),
                    });
                }

                ctx.draw(&Points {
                    coords: &[(
                        self.data.answer.longitude as f64,
                        self.data.answer.latitude as f64,
                    )],
                    color: Color::LightRed,
                });
            })
            .block(
                Block::bordered()
                    .title(format!(" Results {} ", self.data.number))
                    .title_alignment(Alignment::Center),
            )
            .render(areas[0], buf);

        let text: Vec<Line> = self
            .data
            .players
            .iter()
            .map(|x| {
                let you = x.id == self.id;

                let style = Style::new();
                let mut line: Line = if you {
                    Span::styled("you", style.italic())
                } else {
                    Span::styled(&self.lobby[x.id].options.user, style)
                }
                .into();

                line.push_span(Span::raw(format!(" - {}", x.points)));
                line
            })
            .collect();

        Paragraph::new(text)
            .block(
                Block::bordered()
                    .title(" Points ")
                    .padding(Padding::left(1))
                    .title_alignment(Alignment::Center)
                    .title_bottom(format!(" {}eady, {}obby ", "[r]".bold(), "[l]".bold())),
            )
            .render(areas[1], buf);
    }
}
