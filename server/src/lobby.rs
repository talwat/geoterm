use shared::Packet;

use crate::{Message, Server, error::Error, round};

pub async fn handler(server: &mut Server, message: Message) -> Result<(), Error> {
    match message {
        Message::Packet(id, packet) => match packet {
            Ok(packet) => match packet {
                Packet::Init { options } => {
                    server.clients[id].options = Some(options.clone());
                    eprintln!("server(client {id}): {options:?}");

                    let lobby = server.lobby().await;
                    let client = &mut server.clients[id];
                    client
                        .write(&Packet::Confirmed { id, options, lobby })
                        .await?;

                    server.broadcast_lobby(id).await;
                }
                Packet::WaitingStatus { ready } => {
                    server.clients[id].ready = ready;
                    server.broadcast_lobby(id).await;

                    if server.clients.iter().all(|x| x.ready) {
                        server.state = round::new(server).await?;
                    }
                }
                _ => server.kick(id, shared::Error::Illegal).await,
            },
            Err(error) => server.kick(id, error).await,
        },
        Message::Connection(socket, address) => {
            server.client(socket, address).await?;
        }
        Message::Quit | Message::GuessingComplete => return Ok(()),
    }

    Ok(())
}
