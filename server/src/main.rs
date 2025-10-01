use std::net::SocketAddr;

use futures::{executor::block_on, future::join_all};
use shared::{LobbyClient, Packet};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
    task::JoinHandle,
};

use crate::{client::Client, error::Error, lobby::match_lobby};

pub mod client;
pub mod error;
pub mod images;
pub mod lobby;

pub enum Message {
    Connection(TcpStream, SocketAddr),
    Packet(usize, Result<Packet, shared::Error>),
    Quit,
}

pub struct Guess {
    coordinates: (f32, f32),
    user: usize,
}

pub enum State {
    Lobby,
    Round {
        number: usize,
        answer: (f32, f32),
        guesses: Vec<Guess>,
    },
}

pub struct Server {
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    listener: JoinHandle<Result<(), Error>>,
    clients: Vec<Client>,
    counter: usize,
    state: State,
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
        let id = self.counter;
        self.counter += 1;

        let client = Client::new(id, self.tx.clone(), socket).await?;
        self.clients.push(client);

        eprintln!("server: new client at {addr:?} with id {id}");

        Ok(())
    }

    pub async fn kick(&mut self, client: usize, error: shared::Error) {
        eprintln!("server (client {client}): removed: {error}");
        if let Some(index) = self.clients.iter().position(|x| x.id == client) {
            self.clients.remove(index);
        };
    }

    pub async fn broadcast(&mut self, packet: &Packet, exclude: Option<usize>) {
        let futures = self
            .clients
            .iter_mut()
            .filter(|client| client.options.is_some() && !exclude.is_some_and(|x| client.id == x))
            .map(|client| client.write(packet));

        join_all(futures).await;
    }

    pub async fn lobby(&mut self) -> Vec<LobbyClient> {
        self.clients
            .iter()
            .filter_map(|x| {
                Some(LobbyClient {
                    id: x.id,
                    ready: x.ready,
                    user: x.options.as_ref()?.user.clone(),
                })
            })
            .collect()
    }

    pub async fn broadcast_lobby(&mut self, id: usize) {
        let lobby = self.lobby().await;
        self.broadcast(
            &Packet::Lobby {
                action: shared::LobbyAction::Join,
                user: id,
                lobby,
            },
            Some(id),
        )
        .await;
    }

    pub async fn new() -> Result<Self, Error> {
        let (tx, rx) = mpsc::channel(8);
        let tcp = TcpListener::bind("127.0.0.1:3000").await?;
        let listener = tokio::spawn(Self::listen(tcp, tx.clone()));

        Ok(Self {
            tx,
            rx,
            clients: Vec::new(),
            counter: 0,
            listener,
            state: State::Lobby,
        })
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        while let Some(message) = self.rx.recv().await {
            if matches!(message, Message::Quit) {
                break;
            }

            match &self.state {
                State::Lobby => match_lobby(self, message).await?,
                State::Round {
                    number,
                    answer,
                    guesses,
                } => todo!(),
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
