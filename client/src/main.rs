use crossterm::event::KeyCode;
use shared::{
    BufferedSplitExt, ClientOptions, LOCALHOST, Packet, PacketReadExt, PacketWriteExt, Reader,
    Writer, lobby::Clients,
};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};

use crate::{
    logic::{Handler, Loading},
    ui::{UI, lobby, results, round},
};

pub mod logic;
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
    lobby: Clients,
    handle: JoinHandle<eyre::Result<()>>,
}

impl Client {
    pub async fn listener(mut reader: Reader, tx: Sender<Message>) -> eyre::Result<()> {
        while let Ok(packet) = reader.read_packet().await {
            tx.send(Message::Packet(packet)).await?;
        }

        tx.send(Message::Quit).await?;
        Ok(())
    }

    pub async fn new(options: ClientOptions) -> eyre::Result<Self> {
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
            lobby,
        };

        Ok(client)
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
    color_eyre::install()?;
    let options = ClientOptions {
        color: shared::Color::Blue,
        user: String::from("bobby"),
    };

    let mut client = Client::new(options).await?;
    let mut terminal = ratatui::init();
    let mut state = State::Lobby(lobby::Lobby {
        id: client.id,
        ready: client.ready,
        clients: client.lobby.clone(),
        username: "bobby".to_string(),
    });

    let mut ui = UI::init(client.tx.clone());
    ui.render(&mut terminal, &state)?;

    'main: loop {
        while let Some(message) = client.rx.recv().await {
            if message == Message::Quit {
                break 'main;
            }

            if message == Message::Resize {
                ui.render(&mut terminal, &state)?;
                continue;
            }

            let result = match &mut state {
                State::Lobby(state) => state.handle(message, &mut client).await,
                State::Loading => Loading::handle(&mut Loading, message, &mut client).await,
                State::Round(round) => round.handle(message, &mut client).await,
                State::Results(results) => results.handle(message, &mut client).await,
            }?;

            match result {
                logic::Result::Quit => break 'main,
                logic::Result::ChangeState(new) => state = new,
                logic::Result::Unhandled => (), // TODO: Handle this actually.
                logic::Result::Continue => (),
            }

            ui.render(&mut terminal, &state)?;
        }
    }

    client.writer.shutdown().await?;
    Ok(())
}
