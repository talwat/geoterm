use bytes::BytesMut;
use crossterm::event::KeyCode;
use shared::{
    Packet, PacketWriteExt,
    image::{HEIGHT, WIDTH, decode},
};

use crate::{
    Client, Message, State,
    ui::{
        lobby::{self, Lobby},
        results::{self, Results},
        round::{self, Round},
    },
};

pub enum Result {
    ChangeState(State),
    Quit,
    Unhandled,
    Continue,
}

pub(crate) trait Handler {
    fn handle(
        &mut self,
        message: Message,
        client: &mut Client,
    ) -> impl Future<Output = eyre::Result<Result>>;
}

impl Handler for Lobby {
    async fn handle(
        &mut self,
        message: crate::Message,
        client: &mut crate::Client,
    ) -> eyre::Result<Result> {
        Ok(match message {
            Message::Ready => {
                self.ready = !self.ready;
                client
                    .writer
                    .write_packet(Packet::WaitingStatus { ready: self.ready })
                    .await?;

                Result::Continue
            }
            Message::Packet(packet) => match packet {
                Packet::RoundLoading { lobby } => {
                    client.lobby = lobby;
                    Result::ChangeState(State::Loading)
                }
                Packet::LobbyEvent { lobby, .. } => {
                    self.clients = lobby;
                    Result::Continue
                }
                _ => Result::Unhandled,
            },
            _ => Result::Unhandled,
        })
    }
}

pub struct Loading;
impl Handler for Loading {
    async fn handle(
        &mut self,
        message: crate::Message,
        _client: &mut crate::Client,
    ) -> eyre::Result<Result> {
        Ok(match message {
            Message::Packet(packet) => match packet {
                Packet::Round { number, image } => {
                    Result::ChangeState(State::Round(round::Round {
                        image_len: image.len(),
                        image: decode(BytesMut::from(image), WIDTH, HEIGHT)?,
                        cursor: (0.0, 0.0),
                        guessed: false,
                        guessing: false,
                        number,
                    }))
                }
                _ => Result::Unhandled,
            },
            _ => Result::Unhandled,
        })
    }
}

impl Handler for Round {
    async fn handle(&mut self, message: Message, client: &mut Client) -> eyre::Result<Result> {
        Ok(match message {
            Message::Key(key) => match key {
                KeyCode::Char('s') | KeyCode::Char(' ') => {
                    self.guessed = true;
                    client
                        .writer
                        .write_packet(Packet::Guess {
                            coordinates: shared::Coordinate {
                                longitude: self.cursor.0,
                                latitude: self.cursor.1,
                            },
                        })
                        .await?;

                    Result::Continue
                }
                KeyCode::Char('g') | KeyCode::Enter => {
                    self.guessing = !self.guessing;
                    Result::Continue
                }
                KeyCode::Up | KeyCode::Char('i') => {
                    self.cursor.1 += 3.0;
                    Result::Continue
                }
                KeyCode::Down | KeyCode::Char('k') => {
                    self.cursor.1 -= 3.0;
                    Result::Continue
                }
                KeyCode::Left | KeyCode::Char('j') => {
                    self.cursor.0 -= 3.0;
                    Result::Continue
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.cursor.0 += 3.0;
                    Result::Continue
                }
                _ => Result::Unhandled,
            },
            Message::Packet(packet) => match packet {
                Packet::LobbyEvent {
                    action,
                    user: _,
                    lobby,
                } => {
                    if action == shared::lobby::Action::Return {
                        Result::ChangeState(State::Lobby(lobby::Lobby {
                            clients: lobby,
                            username: client.options.user.clone(),
                            ready: false,
                            id: client.id,
                        }))
                    } else {
                        Result::Unhandled
                    }
                }
                Packet::Guessed { player: _ } => Result::Continue,
                Packet::Result { results } => {
                    Result::ChangeState(State::Results(results::Results {
                        ready: false,
                        id: client.id,
                        data: results,
                        lobby: client.lobby.clone(),
                    }))
                }
                _ => Result::Unhandled,
            },
            _ => Result::Unhandled,
        })
    }
}

impl Handler for Results {
    async fn handle(&mut self, message: Message, client: &mut Client) -> eyre::Result<Result> {
        Ok(match message {
            Message::Ready => {
                self.ready = !self.ready;
                client
                    .writer
                    .write_packet(Packet::WaitingStatus { ready: self.ready })
                    .await?;
                Result::Continue
            }
            Message::Key(KeyCode::Char('l')) => {
                client.writer.write_packet(Packet::RequestGameEnd).await?;
                Result::Continue
            }
            Message::Packet(packet) => match packet {
                Packet::RoundLoading { lobby } => {
                    client.lobby = lobby;
                    Result::ChangeState(State::Loading)
                }
                Packet::LobbyEvent {
                    action,
                    user: _,
                    lobby,
                } => {
                    if action == shared::lobby::Action::Return {
                        Result::ChangeState(State::Lobby(lobby::Lobby {
                            clients: lobby,
                            username: client.options.user.clone(),
                            ready: false,
                            id: client.id,
                        }))
                    } else {
                        self.lobby = lobby;
                        Result::Continue
                    }
                }
                _ => Result::Unhandled,
            },
            _ => Result::Unhandled,
        })
    }
}
