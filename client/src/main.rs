use std::io::{BufReader, Cursor};

use crossterm::event::{EventStream, KeyEvent};
use futures::StreamExt;
use image::ImageReader;
use image::imageops::FilterType;
use shared::{ClientOptions, FramedSplitExt, Packet, PacketReadExt, PacketWriteExt};
use tokio::{
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub mod renderer;

enum Message {
    Packet(Packet),
    Input(KeyEvent),
}

struct Client {
    _id: usize,
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

    pub async fn new(options: ClientOptions) -> eyre::Result<Self> {
        let (tx, rx) = mpsc::channel(8);
        let stream = TcpStream::connect("127.0.0.1:3000").await?;
        let (mut reader, mut writer) = stream.framed_split();

        writer.write(&Packet::Init { options }).await?;

        let (id, _options, lobby) = match reader.read().await? {
            Packet::Confirmed { id, options, lobby } => (id, options, lobby),
            other => return Err(shared::Error::Illegal(other).into()),
        };

        eprintln!(
            "client: confirmed with id {id}, {} players in lobby.",
            lobby.len()
        );

        Ok(Self {
            _id: id,
            writer,
            rx,
            handle: tokio::spawn(Self::listener(reader, tx.clone())),
            tx,
        })
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

struct UI {
    input: JoinHandle<eyre::Result<()>>,
}

impl UI {
    async fn input(tx: Sender<Message>) -> eyre::Result<()> {
        let mut stream = EventStream::new();

        while let Some(Ok(event)) = stream.next().await {
            match event {
                crossterm::event::Event::Key(key) => tx.send(Message::Input(key)).await?,
                _ => break,
            }
        }

        Ok(())
    }

    pub fn new(tx: Sender<Message>) -> eyre::Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        Ok(UI {
            input: tokio::spawn(Self::input(tx)),
        })
    }
}

impl Drop for UI {
    fn drop(&mut self) {
        self.input.abort();
        crossterm::terminal::disable_raw_mode().unwrap();
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let options = ClientOptions {
        user: String::from("bobby"),
    };

    let mut client = Client::new(options).await?;
    let _ui = UI::new(client.tx.clone())?;

    while let Some(message) = client.rx.recv().await {
        match message {
            Message::Packet(packet) => match packet {
                Packet::RoundLoading => break,
                Packet::Lobby {
                    action,
                    user,
                    lobby,
                } => {
                    dbg!(lobby);
                    dbg!(action);
                    dbg!(user);
                }
                _ => continue,
            },
            Message::Input(input) => match input.code {
                crossterm::event::KeyCode::Char(char) => match char {
                    'r' => {
                        client
                            .writer
                            .write(&Packet::WaitingStatus { ready: true })
                            .await?
                    }
                    _ => continue,
                },
                _ => continue,
            },
        }
    }

    eprintln!("server loading...");
    let (_number, _players, images, text) = loop {
        match client.rx.recv().await.ok_or(shared::Error::Close)? {
            Message::Packet(Packet::Round {
                number,
                players,
                images,
                text,
            }) => break (number, players, images, text),
            Message::Packet(other) => return Err(shared::Error::Illegal(other).into()),
            Message::Input(_) => continue,
        }
    };

    let image = ImageReader::new(BufReader::new(Cursor::new(&images[1])))
        .with_guessed_format()?
        .decode()?;
    let resized = image.resize(120, 90, FilterType::Lanczos3);

    let image = renderer::render(&resized);
    eprintln!("{image}\nstreet: {}", text.street);

    Ok(())
}
