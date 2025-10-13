use crate::{
    ClientOptions, Color, Coordinate, Error, Packet, Player, RoundData,
    lobby::{self, Clients},
};
use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncReadExt};

pub trait Deserialize<R: AsyncRead + Unpin + Send>: Sized {
    fn deserialize(reader: &mut R)
    -> impl std::future::Future<Output = Result<Self, Error>> + Send;
}

impl<R: AsyncRead + Unpin + Send> Deserialize<R> for ClientOptions {
    async fn deserialize(reader: &mut R) -> Result<Self, Error> {
        let color: Color = unsafe { std::mem::transmute(reader.read_u8().await?) };

        let mut user = [0; 16];
        reader.read_exact(&mut user).await?;
        let user = str::from_utf8(&user)?.trim_end_matches('\0').to_owned();

        Ok(Self { color, user })
    }
}

impl<R: AsyncRead + Unpin + Send> Deserialize<R> for lobby::Client {
    async fn deserialize(reader: &mut R) -> Result<Self, Error> {
        let id = reader.read_u32().await? as usize;
        let ready = reader.read_u8().await? != 0;
        let options = ClientOptions::deserialize(reader).await?;
        Ok(crate::lobby::Client { id, ready, options })
    }
}

impl<R: AsyncRead + Unpin + Send> Deserialize<R> for Coordinate {
    async fn deserialize(reader: &mut R) -> Result<Self, Error> {
        let latitude = reader.read_f32().await?;
        let longitude = reader.read_f32().await?;
        Ok(Self {
            latitude,
            longitude,
        })
    }
}

impl<R: AsyncRead + Unpin + Send> Deserialize<R> for Player {
    async fn deserialize(reader: &mut R) -> Result<Self, Error> {
        let id = reader.read_u32().await? as usize;
        let points = reader.read_u32().await? as u64;
        let has_guess = reader.read_u8().await? != 0;
        let guess = if has_guess {
            Some(Coordinate::deserialize(reader).await?)
        } else {
            let mut pad = [0u8; 2];
            reader.read_exact(&mut pad).await?;
            None
        };

        Ok(Self { id, points, guess })
    }
}

impl<R: AsyncRead + Unpin + Send> Deserialize<R> for RoundData {
    async fn deserialize(reader: &mut R) -> Result<Self, Error> {
        let number = reader.read_u32().await? as usize;
        let answer = Coordinate::deserialize(reader).await?;

        let len = reader.read_u32().await? as usize;
        let mut players = Vec::with_capacity(len);
        for _ in 0..len {
            players.push(Player::deserialize(reader).await?);
        }

        Ok(RoundData {
            number,
            answer,
            players,
        })
    }
}

impl<R: AsyncRead + Unpin + Send> Deserialize<R> for Clients {
    async fn deserialize(reader: &mut R) -> Result<Self, Error> {
        let len = reader.read_u32().await? as usize;
        let mut clients = Vec::with_capacity(len);
        for _ in 0..len {
            clients.push(lobby::Client::deserialize(reader).await?);
        }

        Ok(Self::from(clients))
    }
}

impl<R: AsyncRead + Unpin + Send> Deserialize<R> for Packet {
    async fn deserialize(reader: &mut R) -> Result<Self, Error> {
        match reader.read_u8().await? {
            1 => Ok(Self::Init {
                options: ClientOptions::deserialize(reader).await?,
            }),
            2 => Ok(Self::Confirmed {
                id: reader.read_u32().await? as usize,
                options: ClientOptions::deserialize(reader).await?,
                lobby: Clients::deserialize(reader).await?,
            }),
            3 => Ok(Self::LobbyEvent {
                action: unsafe { std::mem::transmute(reader.read_u8().await?) },
                user: reader.read_u32().await? as usize,
                lobby: Clients::deserialize(reader).await?,
            }),
            4 => Ok(Self::WaitingStatus {
                ready: reader.read_u8().await? != 0,
            }),
            5 => Ok(Self::RoundLoading {
                lobby: Clients::deserialize(reader).await?,
            }),
            6 => {
                let number = reader.read_u32().await? as usize;
                let len = reader.read_u32().await? as usize;

                let mut buf = vec![0u8; len];
                reader.read_exact(&mut buf).await?;

                Ok(Self::Round {
                    number,
                    image: Bytes::from(buf),
                })
            }
            7 => Ok(Self::Guess {
                coordinates: Coordinate::deserialize(reader).await?,
            }),
            8 => Ok(Self::Guessed {
                player: reader.read_u32().await? as usize,
            }),
            9 => Ok(Self::Result {
                round: RoundData::deserialize(reader).await?,
            }),
            10 => Ok(Self::ReturnToLobby),
            tag => Err(Error::Unknown(tag)),
        }
    }
}
