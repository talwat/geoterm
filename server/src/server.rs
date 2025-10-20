use std::{
    net::SocketAddr,
    ops::{Index, IndexMut},
};

use futures::future::join_all;
use shared::{LOCALHOST, Packet, RoundData};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::mpsc,
    task::JoinHandle,
};

use crate::{Message, client::Client, error::Error, lobby, round};

#[derive(Debug, PartialEq)]
pub enum State {
    Lobby,
    Round(RoundData),
    Results(RoundData),
}

pub struct Server {
    pub clients: Vec<Client>,
    pub state: State,
    pub tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    listener: JoinHandle<Result<(), Error>>,
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

    pub async fn return_to_lobby(&mut self, id: usize) {
        self.state = State::Lobby;
        self.broadcast_lobby(id, shared::lobby::Action::Return)
            .await;
    }

    pub async fn verify(&mut self, id: usize) {
        if self.clients.iter().filter(|x| x.initialized()).count() < 2 {
            eprintln!("server: not enough players, returning to lobby...");
            self.return_to_lobby(id).await;
        }
    }

    pub async fn soft_kick(&mut self, client: usize) -> Result<(), Error> {
        eprintln!("server(client {client}): soft quit");
        self[client].options = None;
        self.verify(client).await;

        if self.state == State::Lobby && self.ready() {
            self.state = round::new(self, None).await?;
        } else {
            self.verify(client).await;
        }

        Ok(())
    }

    pub async fn kick(&mut self, client: usize, error: shared::Error) -> Result<(), Error> {
        eprintln!("server(client {client}): removed: {error}");
        if let Some(index) = self.clients.iter().position(|x| x.id == client) {
            self.clients.remove(index);
        };

        if self.state == State::Lobby {
            self.broadcast_lobby(client, shared::lobby::Action::Leave)
                .await;

            if self.ready() {
                self.state = round::new(self, None).await?;
            }
        } else {
            self.verify(client).await;
        }

        Ok(())
    }

    pub async fn broadcast(&mut self, packet: &Packet, exclude: Option<usize>) {
        let futures = self
            .clients
            .iter_mut()
            .filter(|client| client.initialized() && !exclude.is_some_and(|x| client.id == x))
            .map(|client| client.write(packet.clone()));

        join_all(futures).await;
    }

    pub fn ready(&self) -> bool {
        let ready = self.clients.iter().filter(|x| x.ready).count();
        ready >= 2 && ready == self.clients.iter().filter(|x| x.initialized()).count()
    }

    pub async fn lobby(&mut self) -> shared::lobby::Clients {
        let inner: Vec<_> = self
            .clients
            .iter()
            .filter_map(|x| {
                if !x.initialized() {
                    return None;
                }
                Some(shared::lobby::Client {
                    id: x.id,
                    ready: x.ready,
                    options: x.options.clone()?,
                })
            })
            .collect();

        shared::lobby::Clients::from(inner)
    }

    pub async fn broadcast_lobby(&mut self, id: usize, action: shared::lobby::Action) {
        let clients = self.lobby().await;
        let exclude = if action == shared::lobby::Action::Return {
            None
        } else {
            Some(id)
        };

        self.broadcast(
            &Packet::LobbyEvent {
                action,
                user: id,
                lobby: clients,
            },
            exclude,
        )
        .await;
    }

    pub async fn new() -> Result<Self, Error> {
        let (tx, rx) = mpsc::channel(8);
        let tcp = TcpListener::bind(LOCALHOST).await?;
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
                        let mut round = round.clone();
                        round::results(&mut round);

                        self.state = State::Results(round.clone());
                        self.broadcast(&Packet::Result { round }, None).await;
                        eprintln!("server: round finished, showing results");
                    }
                    Message::Connection(mut stream, _addr) => stream.shutdown().await?,
                    Message::Packet(id, packet) => match packet {
                        Ok(Packet::RequestGameEnd) => {
                            eprintln!("server(client {id}): return to lobby");
                            self.return_to_lobby(id).await;
                        }
                        Ok(Packet::SoftQuit) => self.soft_kick(id).await?,
                        Ok(Packet::Guess { coordinates }) => {
                            eprintln!("server(client {id}): guessed at {coordinates:?}");
                            round[id].guess = Some(coordinates);
                            if round.players.iter().all(|x| x.guess.is_some()) {
                                self.tx.send(Message::GuessingComplete).await?;
                            }

                            self.broadcast(&Packet::Guessed { player: id }, Some(id))
                                .await;
                        }
                        Ok(other) => self.kick(id, shared::Error::Illegal(other)).await?,
                        Err(error) => self.kick(id, error).await?,
                    },
                    Message::Quit => return Ok(()),
                },
                State::Results(round) => match message {
                    Message::Packet(id, packet) => match packet {
                        Ok(Packet::RequestGameEnd) => {
                            eprintln!("server(client {id}): returning to lobby...");
                            eprintln!("-> WARNING: packet RequestGameEnd was used.");
                            self.return_to_lobby(id).await;
                        }
                        Ok(Packet::WaitingStatus { ready }) => {
                            if !ready {
                                eprintln!("server(client {id}): returning to lobby...");
                                self.return_to_lobby(id).await;
                                continue;
                            }

                            eprintln!("server(client {id}): ready");
                            let round = round.clone();

                            self[id].ready = ready;
                            if self.ready() {
                                eprintln!("server: all ready, starting new round");
                                self.state = round::new(self, Some(&round)).await?;
                            }
                        }
                        Ok(other) => self.kick(id, shared::Error::Illegal(other)).await?,
                        Err(error) => self.kick(id, error).await?,
                    },
                    Message::Connection(mut stream, _addr) => stream.shutdown().await?,
                    Message::Quit | Message::GuessingComplete => return Ok(()),
                },
            }
        }

        Ok(())
    }
}
