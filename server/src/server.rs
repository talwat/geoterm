use std::{
    net::SocketAddr,
    ops::{Index, IndexMut},
};

use futures::future::join_all;
use shared::{LobbyAction, LobbyClient, Packet, Round};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
    task::JoinHandle,
};

use crate::{Message, client::Client, error::Error, lobby};

pub enum State {
    Lobby,
    Round(Round),
    Results(Round),
}

pub struct Server {
    pub tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    listener: JoinHandle<Result<(), Error>>,
    pub clients: Vec<Client>,
    pub state: State,
    id_counter: usize,
}

impl IndexMut<usize> for Server {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.clients
            .iter_mut()
            .find(|x| x.id == index)
            .expect(&format!("couldn't find client {index}"))
    }
}

impl Index<usize> for Server {
    type Output = Client;

    fn index(&self, index: usize) -> &Self::Output {
        self.clients
            .iter()
            .find(|x| x.id == index)
            .expect(&format!("couldn't find client {index}"))
    }
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
        let id = self.id_counter;
        self.id_counter += 1;

        let client = Client::new(id, self.tx.clone(), socket).await?;
        self.clients.push(client);

        eprintln!("server: new client at {addr:?} with id {id}");

        Ok(())
    }

    pub async fn kick(&mut self, client: usize, error: shared::Error) {
        eprintln!("server(client {client}): removed: {error}");
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

    pub async fn broadcast_lobby(&mut self, id: usize, action: LobbyAction) {
        let lobby = self.lobby().await;
        self.broadcast(
            &Packet::Lobby {
                action,
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
            id_counter: 0,
            listener,
            state: State::Lobby,
        })
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        while let Some(message) = self.rx.recv().await {
            if matches!(message, Message::Quit) {
                break;
            }

            match &mut self.state {
                State::Lobby => lobby::handler(self, message).await?,
                State::Round(round) => match message {
                    Message::GuessingComplete => {
                        let round = round.clone();
                        // TODO: Actually calculate something...
                        self.broadcast(&Packet::Result { round }, None).await;
                    }
                    Message::Connection(_stream, _addr) => return Err(Error::InSession),
                    Message::Packet(id, packet) => match packet {
                        Ok(Packet::Guess { coordinates }) => {
                            round[id].guess = Some(coordinates);
                            if round.players.iter().all(|x| x.guess.is_some()) {
                                self.tx.send(Message::GuessingComplete).await?;
                            }
                        }

                        // TODO: Is it really the best idea to just kick anyone who sends an illegal package?
                        Ok(other) => self.kick(id, shared::Error::Illegal(other)).await,
                        Err(error) => self.kick(id, error).await,
                    },
                    Message::Quit => return Ok(()),
                },
                State::Results { .. } => todo!(),
            }
        }

        Ok(())
    }
}
