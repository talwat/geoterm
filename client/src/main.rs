use std::io::Cursor;

use shared::{ClientOptions, FramedSplitExt, LobbyClient, Packet, PacketReadExt, PacketWriteExt};
use tokio::{
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::ui::{UI, lobby, round};

pub mod ui;

#[derive(Debug)]
pub enum Message {
    Quit,
    Ready,
    Packet(Packet),
}

struct Client {
    ready: bool,
    id: usize,
    writer: FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>,
    rx: Receiver<Message>,
    tx: Sender<Message>,
    handle: JoinHandle<eyre::Result<()>>,
}

impl Client {
    pub async fn listener(
        mut reader: FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
        tx: Sender<Message>,
    ) -> eyre::Result<()> {
        while let Ok(packet) = reader.read().await {
            tx.send(Message::Packet(packet)).await?;
        }

        Ok(())
    }

    pub async fn new(options: ClientOptions) -> eyre::Result<(Self, Vec<LobbyClient>)> {
        let (tx, rx) = mpsc::channel(8);
        let stream = TcpStream::connect("127.0.0.1:4000").await?;
        let (mut reader, mut writer) = stream.framed_split();

        writer.write(&Packet::Init { options }).await?;

        let (id, .., lobby) = match reader.read().await? {
            Packet::Confirmed { id, options, lobby } => (id, options, lobby),
            other => return Err(shared::Error::Illegal(other).into()),
        };

        let client = Self {
            ready: false,
            id,
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
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let options = ClientOptions {
        user: String::from("bobby"),
    };

    let (mut client, lobby) = Client::new(options).await?;
    let mut terminal = ratatui::init();
    let mut state = State::Lobby(lobby::Lobby {
        id: client.id,
        ready: client.ready,
        clients: lobby,
        username: "bobby".to_string(),
    });

    let mut ui = UI::init(client.tx.clone());
    ui.render(&mut terminal, &state)?;

    'main: loop {
        while let Some(message) = client.rx.recv().await {
            match &mut state {
                State::Lobby(lobby) => match message {
                    Message::Quit => break 'main,
                    Message::Ready => {
                        lobby.ready = !lobby.ready;
                        client
                            .writer
                            .write(&Packet::WaitingStatus { ready: lobby.ready })
                            .await?;
                    }
                    Message::Packet(packet) => match packet {
                        Packet::RoundLoading => state = State::Loading,
                        Packet::Lobby { clients, .. } => lobby.clients = clients,
                        _ => continue,
                    },
                },
                State::Loading => match message {
                    Message::Quit => break 'main,
                    Message::Ready => {}
                    Message::Packet(packet) => match packet {
                        Packet::Round {
                            number,
                            text,
                            images,
                        } => {
                            state = State::Round(round::Round {
                                images: images.map(|x| {
                                    image::ImageReader::new(Cursor::new(x))
                                        .with_guessed_format()
                                        .unwrap()
                                        .decode()
                                        .unwrap()
                                        .to_rgb8()
                                }),
                                street: text.street,
                                guessed: false,
                                number,
                            })
                        }
                        _ => continue,
                    },
                },
                State::Round(round) => todo!(),
            }

            ui.render(&mut terminal, &state)?;
        }
    }

    Ok(())
}
