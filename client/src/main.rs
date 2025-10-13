use bytes::BytesMut;
use crossterm::event::KeyCode;
use shared::{
    BufferedSplitExt, ClientOptions, LOCALHOST, Packet, PacketReadExt, PacketWriteExt, Reader,
    Writer,
    image::{HEIGHT, WIDTH, decode},
    lobby::Clients,
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};

use crate::ui::{UI, lobby, results, round};

pub mod ui;

#[derive(Debug, PartialEq)]
pub enum Message {
    Quit,
    Ready,
    Resize,
    Key(KeyCode),
    Packet(Packet),
}

struct Client {
    ready: bool,
    id: usize,
    options: ClientOptions,
    writer: Writer,
    rx: Receiver<Message>,
    tx: Sender<Message>,
    handle: JoinHandle<eyre::Result<()>>,
}

impl Client {
    pub async fn listener(mut reader: Reader, tx: Sender<Message>) -> eyre::Result<()> {
        while let Ok(packet) = reader.read_packet().await {
            tx.send(Message::Packet(packet)).await?;
        }

        Ok(())
    }

    pub async fn new(options: ClientOptions) -> eyre::Result<(Self, Clients)> {
        let (tx, rx) = mpsc::channel(8);
        let stream = TcpStream::connect(LOCALHOST).await?;
        let (mut reader, mut writer) = stream.buffered_split();

        writer
            .write_packet(Packet::Init {
                options: options.clone(),
            })
            .await?;

        let (id, .., lobby) = match reader.read_packet().await? {
            Packet::Confirmed { id, options, lobby } => (id, options, lobby),
            other => return Err(shared::Error::Illegal(other).into()),
        };

        let client = Self {
            ready: false,
            id,
            options,
            writer,
            rx,
            handle: tokio::spawn(Self::listener(reader, tx.clone())),
            tx,
        };

        Ok((client, lobby))
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

pub enum State {
    Loading,
    Lobby(lobby::Lobby),
    Round(round::Round),
    Results(results::Results),
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let options = ClientOptions {
        color: shared::Color::Blue,
        user: String::from("bobby"),
    };

    let (mut client, lobby) = Client::new(options).await?;
    let mut terminal = ratatui::init();
    let mut state = State::Lobby(lobby::Lobby {
        id: client.id,
        ready: client.ready,
        clients: lobby.clone(),
        username: "bobby".to_string(),
    });

    let mut ui = UI::init(client.tx.clone());
    ui.render(&mut terminal, &state)?;

    let mut players: Clients = lobby;

    'main: loop {
        while let Some(message) = client.rx.recv().await {
            if message == Message::Quit {
                break 'main;
            }

            if message == Message::Resize {
                ui.render(&mut terminal, &state)?;
                continue;
            }

            match &mut state {
                State::Lobby(lobby_state) => match message {
                    Message::Ready => {
                        lobby_state.ready = !lobby_state.ready;
                        client
                            .writer
                            .write_packet(Packet::WaitingStatus {
                                ready: lobby_state.ready,
                            })
                            .await?;
                    }
                    Message::Packet(packet) => match packet {
                        Packet::RoundLoading { lobby } => {
                            players = lobby;
                            state = State::Loading
                        }
                        Packet::LobbyEvent { lobby, .. } => lobby_state.clients = lobby,
                        _ => continue,
                    },
                    _ => continue,
                },
                State::Loading => match message {
                    Message::Packet(packet) => match packet {
                        Packet::Round { number, image } => {
                            state = State::Round(round::Round {
                                image_len: image.len(),
                                image: decode(BytesMut::from(image), WIDTH, HEIGHT)?,
                                cursor: (0.0, 0.0),
                                guessed: false,
                                guessing: false,
                                number,
                            })
                        }
                        _ => continue,
                    },
                    _ => continue,
                },
                State::Round(round) => match message {
                    Message::Key(key) => match key {
                        KeyCode::Char(x) => match x {
                            'g' => round.guessing = !round.guessing,
                            ' ' => {
                                round.guessed = true;
                                client
                                    .writer
                                    .write_packet(Packet::Guess {
                                        coordinates: shared::Coordinate {
                                            longitude: round.cursor.0,
                                            latitude: round.cursor.1,
                                        },
                                    })
                                    .await?;
                            }
                            _ => continue,
                        },
                        KeyCode::Up => round.cursor.1 += 3.0,
                        KeyCode::Down => round.cursor.1 -= 3.0,
                        KeyCode::Left => round.cursor.0 -= 3.0,
                        KeyCode::Right => round.cursor.0 += 3.0,
                        _ => continue,
                    },
                    Message::Packet(packet) => match packet {
                        Packet::LobbyEvent {
                            action,
                            user: _,
                            lobby,
                        } => {
                            if action == shared::lobby::Action::Return {
                                state = State::Lobby(lobby::Lobby {
                                    clients: lobby,
                                    username: client.options.user.clone(),
                                    ready: false,
                                    id: client.id,
                                });
                            }
                        }
                        Packet::Guessed { player: _ } => {}
                        Packet::Result { round } => {
                            state = State::Results(results::Results {
                                data: round,
                                lobby: players.clone(),
                            });

                            break;
                        }
                        _ => continue,
                    },
                    _ => continue,
                },
                State::Results(_results) => match message {
                    _ => continue,
                },
            }

            ui.render(&mut terminal, &state)?;
        }
    }

    Ok(())
}
