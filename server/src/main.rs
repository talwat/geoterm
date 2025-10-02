use std::net::SocketAddr;

use futures::executor::block_on;
use shared::Packet;
use tokio::net::TcpStream;

use crate::{error::Error, server::Server};

pub mod client;
pub mod error;
pub mod images;
pub mod lobby;
pub mod round;
pub mod server;

pub enum Message {
    Connection(TcpStream, SocketAddr),
    Packet(usize, Result<Packet, shared::Error>),
    GuessingComplete,
    Quit,
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
