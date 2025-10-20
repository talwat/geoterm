use shared::Packet;

use crate::{Message, Server, error::Error, round};

pub async fn handler(server: &mut Server, message: Message) -> Result<(), Error> {
    match message {
        Message::Packet(id, packet) => match packet {
            Ok(Packet::Init { options }) => {
                server[id].options = Some(options.clone());
                eprintln!("server(client {id}): {options:?}");

                let lobby = server.lobby().await;
                let client = &mut server[id];
                client
                    .write(Packet::Confirmed { id, options, lobby })
                    .await?;

                server
                    .broadcast_lobby(id, shared::lobby::Action::Join)
                    .await;
            }
            Ok(Packet::WaitingStatus { ready }) => {
                server[id].ready = ready;
                eprintln!("server(client {id}): ready = {ready}");
                server
                    .broadcast_lobby(id, shared::lobby::Action::Ready)
                    .await;

                if server.ready() {
                    server.state = round::new(server, None).await?;
                }
            }
            Ok(Packet::SoftQuit) => server.soft_kick(id).await?,
            Ok(other) => server.kick(id, shared::Error::Illegal(other)).await?,
            Err(error) => server.kick(id, error).await?,
        },
        Message::Connection(socket, address) => {
            server.client(socket, address).await?;
        }
        Message::Quit | Message::GuessingComplete => return Ok(()),
    }

    Ok(())
}
