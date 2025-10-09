use std::io::Read;

use crate::{
    ClientOptions, Color, Coordinate, Error, Packet, Player, RoundData,
    lobby::{self, Clients},
};
use byteorder::{BigEndian, ReadBytesExt};
use bytes::{Bytes, BytesMut};

pub trait Deserialize: Sized {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error>;
}

impl Deserialize for ClientOptions {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let color: Color = unsafe { std::mem::transmute(reader.read_u8()?) };

        let mut user = [0; 16];
        reader.read_exact(&mut user)?;
        let user = str::from_utf8(&user)?.trim_end_matches('\0').to_owned();

        Ok(Self { color, user })
    }
}

impl Deserialize for lobby::Client {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let id = reader.read_u32::<BigEndian>()? as usize;
        let ready = reader.read_u8()? != 0;
        let options = ClientOptions::deserialize(reader)?;
        Ok(crate::lobby::Client { id, ready, options })
    }
}

impl Deserialize for Coordinate {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let latitude = reader.read_f32::<BigEndian>()?;
        let longitude = reader.read_f32::<BigEndian>()?;
        Ok(Self {
            longitude,
            latitude,
        })
    }
}

impl Deserialize for Player {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let id = reader.read_u32::<BigEndian>()? as usize;
        let points = reader.read_u32::<BigEndian>()? as u64;
        let has_guess = reader.read_u8()? != 0;
        let guess = if has_guess {
            Some(Coordinate::deserialize(reader)?)
        } else {
            let mut pad = [0u8; 2];
            reader.read_exact(&mut pad)?;

            None
        };

        Ok(Self { guess, points, id })
    }
}

impl Deserialize for RoundData {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let number = reader.read_u32::<BigEndian>()? as usize;
        let answer = Coordinate::deserialize(reader)?;

        let len = reader.read_u32::<BigEndian>()? as usize;
        let players = (0..len)
            .filter_map(|_| Player::deserialize(reader).ok())
            .collect();

        Ok(RoundData {
            number,
            answer,
            players,
        })
    }
}

impl Deserialize for Clients {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let len = reader.read_u32::<BigEndian>()? as usize;
        let clients: Vec<_> = (0..len)
            .filter_map(|_| lobby::Client::deserialize(reader).ok())
            .collect();

        Ok(Self::from(clients))
    }
}

impl Deserialize for Packet {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        match reader.read_u8()? {
            0 => Ok(Self::Init {
                options: ClientOptions::deserialize(reader)?,
            }),
            1 => Ok(Self::Confirmed {
                id: reader.read_u32::<BigEndian>()? as usize,
                options: ClientOptions::deserialize(reader)?,
                lobby: Clients::deserialize(reader)?,
            }),
            2 => Ok(Self::LobbyEvent {
                action: unsafe { std::mem::transmute(reader.read_u8()?) },
                user: reader.read_u32::<BigEndian>()? as usize,
                lobby: Clients::deserialize(reader)?,
            }),
            3 => Ok(Self::WaitingStatus {
                ready: reader.read_u8()? != 0,
            }),
            4 => Ok(Self::RoundLoading {
                lobby: Clients::deserialize(reader)?,
            }),
            5 => {
                let number = reader.read_u32::<BigEndian>()? as usize;
                let len = reader.read_u32::<BigEndian>()? as usize;

                let mut buf = vec![0u8; len];
                reader.read_to_end(&mut buf)?;

                Ok(Self::Round {
                    number,
                    image: Bytes::from(buf),
                })
            }
            6 => Ok(Self::Guess {
                coordinates: crate::Coordinate::deserialize(reader)?,
            }),
            7 => Ok(Self::Guessed {
                player: reader.read_u32::<BigEndian>()? as usize,
            }),
            8 => Ok(Self::Result {
                round: RoundData::deserialize(reader)?,
            }),
            9 => Ok(Self::ReturnToLobby),
            id => Err(Error::Unknown(id)),
        }
    }
}
