use shared::{LobbyAction, Packet};

use crate::{Message, Server, error::Error, round};

pub async fn handler(server: &mut Server, message: Message) -> Result<(), Error> {
    match message {
        Message::Packet(id, packet) => match packet {
            Ok(Packet::Init { options }) => {
                server.clients[id].options = Some(options.clone());
                eprintln!("server(client {id}): {options:?}");

                let lobby = server.lobby().await;
                let client = &mut server.clients[id];
                client
                    .write(&Packet::Confirmed { id, options, lobby })
                    .await?;

                server.broadcast_lobby(id, LobbyAction::Join).await;
            }
            Ok(Packet::WaitingStatus { ready }) => {
                server.clients[id].ready = ready;
                eprintln!("server(client {id}): ready = {ready}");
                server.broadcast_lobby(id, LobbyAction::Ready).await;

                let ready = server.clients.iter().filter(|x| x.ready).count();
                if ready >= 2 && ready == server.clients.len() {
                    server.state = round::new(server).await?;
                }
            }
            Ok(other) => server.kick(id, shared::Error::Illegal(other)).await,
            Err(error) => server.kick(id, error).await,
        },
        Message::Connection(socket, address) => {
            server.client(socket, address).await?;
        }
        Message::Quit | Message::GuessingComplete => return Ok(()),
    }

    Ok(())
}
