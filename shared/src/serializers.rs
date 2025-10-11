use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::{ClientOptions, Coordinate, Error, Packet, Player, RoundData, lobby};

trait ToFixed<const LEN: usize> {
    fn fixed(&self) -> [u8; LEN];
}

impl<const LEN: usize> ToFixed<LEN> for String {
    fn fixed(&self) -> [u8; LEN] {
        let mut writer = [0u8; LEN];
        let slice = self.as_bytes();
        let len = slice.len().min(LEN);
        writer[..len].copy_from_slice(&slice[..len]);

        writer
    }
}

pub trait Serialize<W: AsyncWrite + Unpin + Send> {
    fn serialize(
        &self,
        writer: &mut W,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;
}

impl<W: AsyncWrite + Unpin + Send> Serialize<W> for ClientOptions {
    async fn serialize(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_u8(self.color as u8).await?;

        let user: [u8; 16] = self.user.fixed();
        writer.write_all(&user).await?;

        Ok(())
    }
}

impl<W: AsyncWrite + Unpin + Send> Serialize<W> for lobby::Client {
    async fn serialize(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_u32(self.id as u32).await?;
        writer.write_u8(self.ready as u8).await?;
        self.options.serialize(writer).await?;

        Ok(())
    }
}

impl<W: AsyncWrite + Unpin + Send> Serialize<W> for lobby::Clients {
    async fn serialize(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_u32(self.len() as u32).await?;
        for client in self {
            client.serialize(writer).await?;
        }
        Ok(())
    }
}

impl<W: AsyncWrite + Unpin + Send> Serialize<W> for Coordinate {
    async fn serialize(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_f32(self.latitude).await?;
        writer.write_f32(self.longitude).await?;
        Ok(())
    }
}

impl<W: AsyncWrite + Unpin + Send> Serialize<W> for Player {
    async fn serialize(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_u32(self.id as u32).await?;
        writer.write_u32(self.points as u32).await?;

        writer.write_u8(self.guess.is_some() as u8).await?;
        if let Some(guess) = self.guess {
            guess.serialize(writer).await?;
        } else {
            writer.write_all(&[0; 2]).await?;
        }

        Ok(())
    }
}

impl<W: AsyncWrite + Unpin + Send> Serialize<W> for RoundData {
    async fn serialize(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_u32(self.number as u32).await?;
        self.answer.serialize(writer).await?;

        writer.write_u32(self.players.len() as u32).await?;
        for player in &self.players {
            player.serialize(writer).await?;
        }

        Ok(())
    }
}

impl<W: AsyncWrite + Unpin + Send> Serialize<W> for Packet {
    async fn serialize(&self, writer: &mut W) -> Result<(), Error> {
        let id = self.id();
        writer.write_u8(id).await?;

        match self {
            Packet::Init { options } => options.serialize(writer).await?,
            Packet::Confirmed { id, options, lobby } => {
                writer.write_u32(*id as u32).await?;
                options.serialize(writer).await?;
                lobby.serialize(writer).await?;
            }
            Packet::LobbyEvent {
                action,
                user,
                lobby,
            } => {
                writer.write_u8(*action as u8).await?;
                writer.write_u32(*user as u32).await?;
                lobby.serialize(writer).await?;
            }
            Packet::WaitingStatus { ready } => {
                writer.write_u8(*ready as u8).await?;
            }
            Packet::RoundLoading { lobby } => {
                lobby.serialize(writer).await?;
            }
            Packet::Round { number, image } => {
                writer.write_u32(*number as u32).await?;
                writer.write_u32(image.len() as u32).await?;
                writer.write_all(image).await?;
            }
            Packet::Guess { coordinates } => {
                coordinates.serialize(writer).await?;
            }
            Packet::Guessed { player } => {
                writer.write_u32(*player as u32).await?;
            }
            Packet::Result { round } => {
                round.serialize(writer).await?;
            }
            Packet::ReturnToLobby => {}
        }

        writer.flush().await?;
        Ok(())
    }
}
