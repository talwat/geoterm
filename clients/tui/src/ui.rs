use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use futures::StreamExt;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Flex, Layout, Rect},
    widgets::Widget,
};
use tokio::{sync::mpsc::Sender, task::JoinHandle};

use crate::{Message, State};

pub mod loading;
pub mod lobby;
pub mod results;
pub mod round;

pub fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

pub struct UI {
    input: JoinHandle<eyre::Result<()>>,
    _tx: Sender<crate::Message>,
}

impl UI {
    fn draw(&self, frame: &mut Frame, state: &State) {
        let area = frame.area();
        let buf = frame.buffer_mut();

        match state {
            State::Lobby(lobby) => lobby.render(area, buf),
            State::Round(round) => {
                if round.guessed {
                    loading::render(area, buf, "waiting for others to guess...")
                } else {
                    round.render(area, buf)
                }
            }
            State::Loading => loading::render(area, buf, "loading..."),
            State::Results(results) => results.render(area, buf),
        }
    }

    pub fn render(&mut self, terminal: &mut DefaultTerminal, state: &State) -> eyre::Result<()> {
        terminal.draw(|frame| self.draw(frame, state))?;
        Ok(())
    }

    async fn input(tx: Sender<crate::Message>) -> eyre::Result<()> {
        let mut stream = event::EventStream::new();
        while let Some(Ok(event)) = stream.next().await {
            let message = match event {
                Event::Key(key) => match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char('r') => Message::Ready,
                    key => Message::Key(key),
                },
                Event::Resize(..) => Message::Resize,
                _ => continue,
            };

            tx.send(message).await?;
        }

        tx.send(Message::Quit).await?;
        Ok(())
    }

    pub fn init(tx: Sender<crate::Message>) -> Self {
        Self {
            _tx: tx.clone(),
            input: tokio::spawn(Self::input(tx)),
        }
    }
}

impl Drop for UI {
    fn drop(&mut self) {
        self.input.abort();
        ratatui::restore();
    }
}
