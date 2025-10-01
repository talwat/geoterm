use std::{net::SocketAddr, str::Utf8Error};

use futures::executor::block_on;
use shared::{ClientOptions, FramedSplitExt, Packet, PacketReadExt, PacketWriteExt};
use tokio::{
    io::{self, AsyncWriteExt},
    net::{
        TcpListener, TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::mpsc::{self, error::SendError},
    task::JoinHandle,
};
use tokio_util::codec::{self, FramedRead, FramedWrite, LengthDelimitedCodec};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("utf8 encoding/decoding failed")]
    Utf8(#[from] Utf8Error),

    #[error("sending internal message failed")]
    Send(#[from] SendError<Message>),

    #[error("io failure")]
    Io(#[from] io::Error),

    #[error("packet error")]
    Packet(#[from] shared::Error),
}

pub enum Message {
    Connection(TcpStream, SocketAddr),
    Packet(usize, Result<Packet, shared::Error>),
    Quit,
}

struct Client {
    pub id: usize,
    pub options: Option<ClientOptions>,
    writer: FramedWrite<OwnedWriteHalf, codec::LengthDelimitedCodec>,
    handle: JoinHandle<Result<(), Error>>,

    #[allow(dead_code)]
    tx: mpsc::Sender<Message>,
}

impl Client {
    pub async fn listener(
        id: usize,
        tx: mpsc::Sender<Message>,
        mut reader: FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
    ) -> Result<(), Error> {
        loop {
            let packet = reader.read().await;
            let is_err = packet.is_err();
            tx.send(Message::Packet(id, packet)).await?;

            if is_err {
                break;
            };
        }

        Ok(())
    }

    pub async fn write(&mut self, packet: &Packet) -> Result<(), Error> {
        self.writer.write(packet).await?;
        Ok(())
    }

    pub async fn new(
        id: usize,
        tx: mpsc::Sender<Message>,
        socket: TcpStream,
    ) -> Result<Self, Error> {
        let (reader, writer) = socket.framed_split();
        let handle = tokio::spawn(Self::listener(id, tx.clone(), reader));

        Ok(Self {
            handle,
            id,
            tx,
            options: None,
            writer,
        })
    }

    pub async fn close(self) {
        self.handle.abort();
        let _ = self.writer.into_inner().shutdown().await;
        eprintln!("server (client {}): closed", self.id);
    }
}

struct Server {
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    listener: JoinHandle<Result<(), Error>>,
    clients: Vec<Client>,
}

impl Drop for Server {
    fn drop(&mut self) {
        self.listener.abort();
    }
}

impl Server {
    pub async fn listen(listener: TcpListener, tx: mpsc::Sender<Message>) -> Result<(), Error> {
        eprintln!("server: listening on {}", listener.local_addr()?);
        while let Ok((stream, addr)) = listener.accept().await {
            tx.send(Message::Connection(stream, addr)).await.unwrap()
        }
        Ok(())
    }

    pub async fn client(&mut self, socket: TcpStream, addr: SocketAddr) -> Result<(), Error> {
        let id = self.clients.last().and_then(|x| Some(x.id)).unwrap_or(0);
        let client = Client::new(id, self.tx.clone(), socket).await?;
        self.clients.push(client);

        eprintln!("server: new client at {addr:?} with id {id}");

        Ok(())
    }

    pub async fn kick(&mut self, client: usize, error: shared::Error) {
        eprintln!("server (client {client}): removed: {error}");
        self.clients.remove(client).close().await;
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

                            eprintln!("server (client {id}): {options:?}");
                            client.write(&Packet::Confirmed { id, options }).await?;
                        }
                        Packet::Confirmed { id, options: _ } => {
                            self.kick(id, shared::Error::Illegal).await
                        }
                    },
                    Err(error) => self.kick(id, error).await,
                },
                Message::Connection(socket, address) => {
                    self.client(socket, address).await?;
                }
                Message::Quit => break,
            }
        }

        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    let mut server = Server::new().await?;
    let tx = server.tx.clone();
    ctrlc::set_handler(move || block_on(tx.send(Message::Quit)).unwrap()).unwrap();

    server.run().await?;
    for client in server.clients.drain(..) {
        client.close().await;
    }

    Ok(())
}
