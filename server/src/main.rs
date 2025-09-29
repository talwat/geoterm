use std::net::SocketAddr;

use tokio::{
    io::{self, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf}, TcpListener, TcpStream
    },
    sync::mpsc,
    task::JoinHandle,
};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("IO failure")]
    Io(#[from] io::Error),
}

pub enum Message {
    Connection(TcpStream, SocketAddr),
    Packet(usize),
}

struct Client {
    pub id: usize,
    pub user: String,
    writer: OwnedWriteHalf,
    handle: JoinHandle<()>,

    #[allow(dead_code)]
    tx: mpsc::Sender<Message>,
}

impl Client {
    pub async fn listener(tx: mpsc::Sender<Message>, reader: OwnedReadHalf) {
        todo!()
    }

    // TODO: Put packet instead of just... `data`. 
    pub async fn write(&mut self, data: u8) -> Result<(), Error> {
        Ok(self.writer.write_u8(data).await?)
    }

    pub fn new(id: usize, tx: mpsc::Sender<Message>, socket: TcpStream) -> Self {
        let (reader, writer) = socket.into_split();
        let handle = tokio::spawn(Self::listener(tx.clone(), reader));

        Self {
            handle,
            id,
            tx,
            user: String::new(),
            writer,
        }
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

    pub fn client(&mut self, socket: TcpStream) {
        let id = self.clients.last().and_then(|x| Some(x.id)).unwrap_or(0);
        let client = Client::new(id, self.tx.clone(), socket);
        self.clients.push(client);
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

    pub async fn run(mut self) -> Result<(), Error> {
        while let Some(action) = self.rx.recv().await {
            match action {
                Message::Packet(client) => {
                    eprintln!("server: packet from {client}");
                }

                Message::Connection(socket, _address) => {
                    self.client(socket);
                }
            }
        }   
        
        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    let server = Server::new().await?;
    server.run().await?;

    Ok(())
}
