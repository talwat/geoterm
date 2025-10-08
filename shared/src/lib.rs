use std::ops::IndexMut;

use futures::{SinkExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::{
    TcpStream,
    tcp::{OwnedReadHalf, OwnedWriteHalf},
};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub type Writer = FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>;
pub type Reader = FramedRead<OwnedReadHalf, LengthDelimitedCodec>;

pub mod image;
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RoundData {
    pub number: usize,
    pub answer: (f32, f32),
    pub players: Vec<Player>,
}

use std::ops::Index;

impl Index<usize> for RoundData {
    type Output = Player;

    fn index(&self, index: usize) -> &Self::Output {
        self.players
            .iter()
            .find(|x| x.id == index)
            .expect("player not found")
    }
}

impl IndexMut<usize> for RoundData {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.players
            .iter_mut()
            .find(|x| x.id == index)
            .expect("player not found")
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct LobbyClient {
    pub id: usize,
    pub ready: bool,
    pub options: ClientOptions,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum LobbyAction {
    Join = 0,
    Return,
    Leave,
    Ready,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum Color {
    Red = 0,
    Yellow,
    Green,
    Blue,
    Magenta,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct ClientOptions {
    pub user: String,
    pub color: Color,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy)]
pub struct Coordinate {
    pub lon: f32,
    pub lat: f32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy)]
pub struct Player {
    pub guess: Option<Coordinate>,
    pub points: u64,
    pub id: usize,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[repr(u8)]
#[serde(tag = "tag")]
pub enum Packet {
    Init {
        options: ClientOptions,
    },
    Confirmed {
        id: usize,
        options: ClientOptions,
        lobby: Vec<LobbyClient>,
    },
    LobbyEvent {
        action: LobbyAction,
        user: usize,
        lobby: Vec<LobbyClient>,
    },
    WaitingStatus {
        ready: bool,
    },
    RoundLoading {
        lobby: Vec<LobbyClient>,
    },
    Round {
        number: usize,
        #[serde(with = "serde_bytes")]
        image: Vec<u8>,
    },
    Guess {
        coordinates: Coordinate,
    },
    Guessed {
        player: usize,
    },
    Result {
        round: RoundData,
    },
    ReturnToLobby,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("socket closed")]
    Close,

    #[error("io failure")]
    Io(#[from] std::io::Error),

    #[error("packet decoding failed")]
    Decode(#[from] rmp_serde::decode::Error),

    #[error("packet encoding failed")]
    Encode(#[from] rmp_serde::encode::Error),

    #[error("unexpected packet: {0:?}")]
    Illegal(Packet),
}

pub trait PacketReadExt {
    fn read(&mut self) -> impl Future<Output = Result<Packet, Error>>;
}

pub trait PacketWriteExt {
    fn write(&mut self, packet: &Packet) -> impl Future<Output = Result<(), Error>>;
}

impl PacketReadExt for Reader {
    async fn read(&mut self) -> Result<Packet, Error> {
        if let Some(bytes) = self.try_next().await? {
            let packet: Packet = rmp_serde::from_slice(&bytes)?;
            Ok(packet)
        } else {
            Err(Error::Close)
        }
    }
}

impl PacketWriteExt for Writer {
    async fn write(&mut self, packet: &Packet) -> Result<(), Error> {
        let bytes = rmp_serde::to_vec(packet)?;

        if bytes.len() < 128 {
            eprintln!("sending: {0:x?}", bytes);
        }
        self.send(bytes.into()).await?;

        Ok(())
    }
}

pub trait FramedSplitExt {
    fn framed_split(self) -> (Reader, Writer);
}

impl FramedSplitExt for TcpStream {
    fn framed_split(self) -> (Reader, Writer) {
        let (reader, writer) = self.into_split();
        let codec = LengthDelimitedCodec::new();

        (
            FramedRead::new(reader, codec.clone()),
            FramedWrite::new(writer, codec),
        )
    }
}
