use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use futures::StreamExt;
use ratatui::{DefaultTerminal, Frame, widgets::Widget};
use tokio::{sync::mpsc::Sender, task::JoinHandle};

use crate::{Message, ui::lobby::Lobby};

pub mod lobby;

pub enum State {
    Lobby(Lobby),
    Round,
}

pub struct UI {
    pub state: State,
    input: JoinHandle<eyre::Result<()>>,
    _tx: Sender<crate::Message>,
}

impl UI {
    fn draw(&self, frame: &mut Frame) {
        match &self.state {
            State::Lobby(lobby) => lobby.render(frame.area(), frame.buffer_mut()),
            State::Round => todo!(),
        }
    }

    pub fn render(&mut self, terminal: &mut DefaultTerminal) -> eyre::Result<()> {
        terminal.draw(|frame| self.draw(frame))?;
        Ok(())
    }

    async fn input(tx: Sender<crate::Message>) -> eyre::Result<()> {
        let mut stream = event::EventStream::new();
        while let Some(Ok(event)) = stream.next().await {
            match event {
                Event::Key(key) => match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char('r') => tx.send(Message::Ready).await?,
                    _ => (),
                },
                _ => (),
            }
        }

        tx.send(Message::Quit).await?;
        Ok(())
    }

    pub fn init(tx: Sender<crate::Message>, state: State) -> Self {
        Self {
            state,
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
