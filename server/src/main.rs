use std::{net::SocketAddr, str::Utf8Error};

use packets::{ClientOptions, Packet};
use tokio::{
    io,
    net::{
        TcpListener, TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::mpsc::{self, error::SendError},
    task::JoinHandle,
};
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Utf8 encoding/decoding failed")]
    Utf8(#[from] Utf8Error),

    #[error("Sending internal message failed")]
    Send(#[from] SendError<Message>),

    #[error("IO failure")]
    Io(#[from] io::Error),

    #[error("Packet error")]
    Packet(#[from] packets::Error),
}

pub enum Message {
    Connection(TcpStream, SocketAddr),
    Packet(usize, Result<Packet, packets::Error>),
}

struct Client {
    pub id: usize,
    pub options: Option<ClientOptions>,
    writer: OwnedWriteHalf,
    handle: JoinHandle<Result<(), Error>>,

    #[allow(dead_code)]
    tx: mpsc::Sender<Message>,
}

impl Client {
    pub async fn listener(
        id: usize,
        tx: mpsc::Sender<Message>,
        mut reader: OwnedReadHalf,
    ) -> Result<(), Error> {
        let mut framed = FramedRead::new(&mut reader, LengthDelimitedCodec::new());
        loop {
            let packet = Packet::read(&mut framed).await;
            let is_err = packet.is_err();
            tx.send(Message::Packet(id, packet)).await?;

            if is_err {
                break;
            };
        }

        Ok(())
    }

    pub async fn write(&mut self, packet: &Packet) -> Result<(), Error> {
        Packet::write(&mut self.writer, packet).await?;
        Ok(())
    }

    pub async fn new(
        id: usize,
        tx: mpsc::Sender<Message>,
        socket: TcpStream,
    ) -> Result<Self, Error> {
        let (reader, writer) = socket.into_split();
        let handle = tokio::spawn(Self::listener(id, tx.clone(), reader));

        Ok(Self {
            handle,
            id,
            tx,
            options: None,
            writer,
        })
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

struct Server {
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    listener: JoinHandle<()>,
    clients: Vec<Client>,
}

impl Server {
    pub async fn listen(listener: TcpListener, tx: mpsc::Sender<Message>) {
        while let Ok((stream, addr)) = listener.accept().await {
            tx.send(Message::Connection(stream, addr)).await.unwrap()
        }
    }

    pub async fn client(&mut self, socket: TcpStream) -> Result<(), Error> {
        let id = self.clients.last().and_then(|x| Some(x.id)).unwrap_or(0);
        let client = Client::new(id, self.tx.clone(), socket).await?;
        self.clients.push(client);

        Ok(())
    }

    pub fn kick(&mut self, client: usize, error: packets::Error) {
        eprintln!("server: client {client} removed: {error}");
        self.clients.remove(client);
    }

    pub async fn new() -> Result<Self, Error> {
        let (tx, rx) = mpsc::channel(8);
        let tcp = TcpListener::bind("127.0.0.1:3000").await?;
        let listener = tokio::spawn(Self::listen(tcp, tx.clone()));

        Ok(Self {
            tx,
            rx,
            clients: Vec::new(),
            listener,
        })
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        while let Some(action) = self.rx.recv().await {
            match action {
                Message::Packet(id, packet) => match packet {
                    Ok(packet) => match packet {
                        Packet::Init { options } => {
                            let client = &mut self.clients[id];
                            client.options = Some(options.clone());
                            client.write(&Packet::Confirmed { id, options }).await?;
                        }
                        Packet::Confirmed { id: _, options: _ } => {
                            self.kick(id, packets::Error::Illegal)
                        }
                    },
                    Err(error) => self.kick(id, error),
                },
                Message::Connection(socket, _address) => {
                    self.client(socket).await?;
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    let mut server = Server::new().await?;
    server.run().await?;
    server.listener.abort();

    Ok(())
}
