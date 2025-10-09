use std::{ops::IndexMut, str::Utf8Error};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::StreamExt;
use tokio::net::{
    TcpStream,
    tcp::{OwnedReadHalf, OwnedWriteHalf},
};
use tokio_util::codec::{Decoder, Encoder, FramedRead, FramedWrite};

pub type Writer = FramedWrite<OwnedWriteHalf, PacketCodec>;
pub type Reader = FramedRead<OwnedReadHalf, PacketCodec>;

pub mod deserializers;
pub mod image;
pub mod lobby;
pub mod serializers;

#[derive(Clone, Debug, PartialEq)]
pub struct RoundData {
    pub number: usize,
    pub answer: Coordinate,
    pub players: Vec<Player>,
}

use std::ops::Index;

use crate::{
    deserializers::Deserialize,
    lobby::{Action, Clients},
    serializers::Serialize,
};

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

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum Color {
    Red = 0,
    Yellow,
    Green,
    Blue,
    Magenta,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClientOptions {
    pub color: Color,
    pub user: String,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Coordinate {
    pub longitude: f32,
    pub latitude: f32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Player {
    pub guess: Option<Coordinate>,
    pub points: u64,
    pub id: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Packet {
    Init {
        options: ClientOptions,
    },
    Confirmed {
        id: usize,
        options: ClientOptions,
        lobby: Clients,
    },
    LobbyEvent {
        action: Action,
        user: usize,
        lobby: Clients,
    },
    WaitingStatus {
        ready: bool,
    },
    RoundLoading {
        lobby: Clients,
    },
    Round {
        number: usize,
        image: Bytes,
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

impl Packet {
    pub fn id(&self) -> u8 {
        match self {
            Packet::Init { .. } => 0,
            Packet::Confirmed { .. } => 1,
            Packet::LobbyEvent { .. } => 2,
            Packet::WaitingStatus { .. } => 3,
            Packet::RoundLoading { .. } => 4,
            Packet::Round { .. } => 5,
            Packet::Guess { .. } => 6,
            Packet::Guessed { .. } => 7,
            Packet::Result { .. } => 8,
            Packet::ReturnToLobby => 9,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("socket closed")]
    Close,

    #[error("io failure")]
    Io(#[from] std::io::Error),

    #[error("utf8 error")]
    Utf8(#[from] Utf8Error),

    #[error("unexpected packet: {0:?}")]
    Illegal(Packet),

    #[error("unknown packet: {0:?}")]
    Unknown(u8),
}

#[derive(Clone)]
pub struct PacketCodec;

impl Decoder for PacketCodec {
    type Item = Packet;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Packet>, Error> {
        if src.is_empty() {
            return Ok(None);
        }

        let packet = Packet::deserialize(&mut src.reader())?;
        Ok(Some(packet))
    }
}

impl Encoder<Packet> for PacketCodec {
    type Error = Error;

    fn encode(&mut self, packet: Packet, dst: &mut BytesMut) -> Result<(), Error> {
        packet.serialize(&mut dst.writer())?;
        Ok(())
    }
}

pub trait PacketReadExt {
    fn read(&mut self) -> impl Future<Output = Result<Packet, Error>> + Send;
}

impl PacketReadExt for Reader {
    async fn read(&mut self) -> Result<Packet, Error> {
        match self.next().await {
            Some(Ok(packet)) => Ok(packet),
            Some(Err(e)) => Err(e),
            None => Err(Error::Close),
        }
    }
}
pub trait FramedSplitExt {
    fn framed_split(self) -> (Reader, Writer);
}

impl FramedSplitExt for TcpStream {
    fn framed_split(self) -> (Reader, Writer) {
        let (reader, writer) = self.into_split();

        let read = PacketCodec;
        let write = PacketCodec;

        let reader = FramedRead::new(reader, read);
        let writer = FramedWrite::new(writer, write);

        (reader, writer)
    }
}
