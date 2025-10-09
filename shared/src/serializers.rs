use std::io::Write;

use crate::{ClientOptions, Coordinate, Error, Packet, Player, RoundData, lobby};
use byteorder::{BigEndian, WriteBytesExt};

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

pub trait Serialize {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error>;
}

impl Serialize for ClientOptions {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_u8(self.color as u8)?;

        let user: [u8; 16] = self.user.fixed();
        writer.write_all(&user)?;

        Ok(())
    }
}

impl Serialize for lobby::Client {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_u32::<BigEndian>(self.id as u32)?;
        writer.write_u8(self.ready as u8)?;
        self.options.serialize(writer)?;

        Ok(())
    }
}

impl Serialize for lobby::Clients {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_u32::<BigEndian>(self.len() as u32)?;
        for client in self {
            client.serialize(writer)?;
        }

        Ok(())
    }
}

impl Serialize for Coordinate {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_f32::<BigEndian>(self.latitude)?;
        writer.write_f32::<BigEndian>(self.longitude)?;

        Ok(())
    }
}

impl Serialize for Player {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_u32::<BigEndian>(self.id as u32)?;
        writer.write_u32::<BigEndian>(self.points as u32)?;

        writer.write_u8(self.guess.is_some() as u8)?;
        if let Some(guess) = self.guess {
            guess.serialize(writer)?;
        } else {
            writer.write(&[0; 2])?;
        }

        Ok(())
    }
}

impl Serialize for RoundData {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_u32::<BigEndian>(self.number as u32)?;
        self.answer.serialize(writer)?;

        writer.write_u32::<BigEndian>(self.players.len() as u32)?;
        for player in &self.players {
            player.serialize(writer)?;
        }

        Ok(())
    }
}

impl Serialize for Packet {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let id = self.id();
        writer.write_u8(id)?;

        match self {
            Packet::Init { options } => options.serialize(writer)?,
            Packet::Confirmed { id, options, lobby } => {
                writer.write_u32::<BigEndian>(*id as u32)?;
                options.serialize(writer)?;
                lobby.serialize(writer)?;
            }
            Packet::LobbyEvent {
                action,
                user,
                lobby,
            } => {
                writer.write_u8(*action as u8)?;
                writer.write_u32::<BigEndian>(*user as u32)?;
                lobby.serialize(writer)?;
            }
            Packet::WaitingStatus { ready } => {
                writer.write_u8(*ready as u8)?;
            }
            Packet::RoundLoading { lobby } => {
                lobby.serialize(writer)?;
            }
            Packet::Round { number, image } => {
                writer.write_u32::<BigEndian>(*number as u32)?;
                writer.write_u32::<BigEndian>(image.len() as u32)?;
                writer.write_all(image)?;
            }
            Packet::Guess { coordinates } => {
                coordinates.serialize(writer)?;
            }
            Packet::Guessed { player } => {
                writer.write_u32::<BigEndian>(*player as u32)?;
            }
            Packet::Result { round } => {
                round.serialize(writer)?;
            }
            Packet::ReturnToLobby => {}
        }

        Ok(())
    }
}
