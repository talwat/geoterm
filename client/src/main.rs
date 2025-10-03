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

use crate::ui::{State, UI, lobby::Lobby};

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
        let stream = TcpStream::connect("127.0.0.1:3000").await?;
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

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let options = ClientOptions {
        user: String::from("bobby"),
    };

    let (mut client, lobby) = Client::new(options).await?;
    let mut terminal = ratatui::init();
    let mut ui = UI::init(
        client.tx.clone(),
        ui::State::Lobby(Lobby {
            id: client.id,
            ready: client.ready,
            clients: lobby,
            username: "bobby".to_string(),
        }),
    );

    ui.render(&mut terminal)?;

    'main: loop {
        while let Some(message) = client.rx.recv().await {
            match &mut ui.state {
                ui::State::Lobby(state) => match message {
                    Message::Quit => break 'main,
                    Message::Ready => {
                        state.ready = !state.ready;
                        client
                            .writer
                            .write(&Packet::WaitingStatus { ready: state.ready })
                            .await?;
                    }
                    Message::Packet(packet) => match packet {
                        Packet::RoundLoading => break,
                        Packet::Lobby { clients, .. } => state.clients = clients,
                        _ => continue,
                    },
                },
                State::Round => todo!(),
            }

            ui.render(&mut terminal)?;
        }
    }

    Ok(())
}
