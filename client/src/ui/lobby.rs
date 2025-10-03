use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};
use shared::LobbyClient;

#[derive(Debug, Default)]
pub struct Lobby {
    pub clients: Vec<LobbyClient>,
    pub username: String,
    pub ready: bool,
    pub id: usize,
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

impl Widget for &Lobby {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let len = self.clients.len() as u16;
        let centered = center(area, Constraint::Length(32), Constraint::Length(4 + len));

        let layout = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(2), Constraint::Fill(1)])
            .split(centered);

        let text: Vec<Line> = self
            .clients
            .iter()
            .map(|x| {
                let you = x.id == self.id;
                let style = Style::new();

                let mut line: Line = if you {
                    Span::styled("- you", style.italic())
                } else {
                    Span::styled(format!("- {}", &x.user), style)
                }
                .into();

                if x.ready || (you && self.ready) {
                    line.spans.push(Span::raw(" (ready)"))
                }

                line
            })
            .collect();

        let block = Block::bordered()
            .title_alignment(Alignment::Center)
            .title(" lobby ".to_string());
        block.render(centered, buf);

        Paragraph::new(format!("user: {}", self.username)).render(layout[0], buf);

        Paragraph::new(format!("geoterm {}", env!("CARGO_PKG_VERSION")))
            .alignment(Alignment::Right)
            .render(layout[0], buf);
        Paragraph::new(text).render(layout[1], buf);
    }
}
