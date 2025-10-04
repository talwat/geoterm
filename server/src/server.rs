use std::{
    net::SocketAddr,
    ops::{Index, IndexMut},
};

use futures::future::join_all;
use shared::{LobbyAction, LobbyClient, Packet, Round};
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

    pub async fn return_to_lobby(&mut self) {
        self.state = State::Lobby;
        self.broadcast(&Packet::ReturnToLobby, None).await;
    }

    pub async fn kick(&mut self, client: usize, error: shared::Error) {
        eprintln!("server(client {client}): removed: {error}");
        if let Some(index) = self.clients.iter().position(|x| x.id == client) {
            self.clients.remove(index);
        };

        if self.state == State::Lobby {
            self.broadcast_lobby(client, LobbyAction::Leave).await;
        } else if self.clients.iter().filter(|x| x.options.is_some()).count() < 2 {
            eprintln!("server: not enough players, returning to lobby...");
            self.return_to_lobby().await;
        }
    }

    pub async fn broadcast(&mut self, packet: &Packet, exclude: Option<usize>) {
        let futures = self
            .clients
            .iter_mut()
            .filter(|client| client.options.is_some() && !exclude.is_some_and(|x| client.id == x))
            .map(|client| client.write(packet));

        join_all(futures).await;
    }

    pub fn ready(&self) -> bool {
        let ready = self.clients.iter().filter(|x| x.ready).count();
        ready >= 2 && ready == self.clients.len()
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
        let clients = self.lobby().await;
        self.broadcast(
            &Packet::Lobby {
                action,
                user: id,
                clients,
            },
            Some(id),
        )
        .await;
    }

    pub async fn new() -> Result<Self, Error> {
        let (tx, rx) = mpsc::channel(8);
        let tcp = TcpListener::bind("127.0.0.1:4000").await?;
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
                        Ok(Packet::Guess { coordinates }) => {
                            eprintln!("server(client {id}): guessed");
                            round[id].guess = Some(coordinates);
                            if round.players.iter().all(|x| x.guess.is_some()) {
                                self.tx.send(Message::GuessingComplete).await?;
                            }

                            self.broadcast(&Packet::Guessed { player: id }, Some(id))
                                .await;
                        }
                        Ok(other) => self.kick(id, shared::Error::Illegal(other)).await,
                        Err(error) => self.kick(id, error).await,
                    },
                    Message::Quit => return Ok(()),
                },
                State::Results(round) => match message {
                    Message::Packet(id, packet) => match packet {
                        Ok(Packet::WaitingStatus { ready }) => {
                            if !ready {
                                eprintln!("server(client {id}): returning to lobby...");
                                self.return_to_lobby().await;
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
                        Ok(other) => self.kick(id, shared::Error::Illegal(other)).await,
                        Err(error) => self.kick(id, error).await,
                    },
                    Message::Connection(mut stream, _addr) => stream.shutdown().await?,
                    Message::Quit | Message::GuessingComplete => return Ok(()),
                },
            }
        }

        Ok(())
    }
}
