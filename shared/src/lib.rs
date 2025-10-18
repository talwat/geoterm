use std::{
    net::{Ipv4Addr, SocketAddrV4},
    ops::IndexMut,
    str::Utf8Error,
};

use bytes::Bytes;
use tokio::{
    io::{BufReader, BufWriter},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
};

pub type Writer = BufWriter<OwnedWriteHalf>;
pub type Reader = BufReader<OwnedReadHalf>;

pub mod deserializers;
pub mod image;
pub mod lobby;
pub mod serializers;

pub const PORT: u16 = 4000;
pub const LOCALHOST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), PORT);

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

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Coordinate {
    /// North-South, like the "y".
    pub latitude: f32,

    /// East-West, like the "x".
    pub longitude: f32,
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
    RequestGameEnd,
    SoftQuit,
}

impl Packet {
    pub fn tag(&self) -> u8 {
        match self {
            Packet::Init { .. } => 1,
            Packet::Confirmed { .. } => 2,
            Packet::LobbyEvent { .. } => 3,
            Packet::WaitingStatus { .. } => 4,
            Packet::RoundLoading { .. } => 5,
            Packet::Round { .. } => 6,
            Packet::Guess { .. } => 7,
            Packet::Guessed { .. } => 8,
            Packet::Result { .. } => 9,
            Packet::RequestGameEnd => 10,
            Packet::SoftQuit => 11,
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

pub trait BufferedSplitExt {
    fn buffered_split(self) -> (Reader, Writer);
}

impl BufferedSplitExt for TcpStream {
    fn buffered_split(self) -> (Reader, Writer) {
        let (reader, writer) = self.into_split();
        (BufReader::new(reader), BufWriter::new(writer))
    }
}

pub trait PacketReadExt {
    fn read_packet(&mut self) -> impl Future<Output = Result<Packet, Error>> + Send;
}

pub trait PacketWriteExt {
    fn write_packet(&mut self, packet: Packet) -> impl Future<Output = Result<(), Error>> + Send;
}

impl PacketWriteExt for Writer {
    async fn write_packet(&mut self, packet: Packet) -> Result<(), Error> {
        packet.serialize(self).await?;
        Ok(())
    }
}

impl PacketReadExt for Reader {
    async fn read_packet(&mut self) -> Result<Packet, Error> {
        Packet::deserialize(self).await
    }
}
