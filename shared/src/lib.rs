use futures::{SinkExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::{
    TcpStream,
    tcp::{OwnedReadHalf, OwnedWriteHalf},
};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct ClientOptions {
    pub user: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[repr(u8)]
#[serde(tag = "tag")]
pub enum Packet {
    Init { options: ClientOptions },
    Confirmed { id: usize, options: ClientOptions },
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

    #[error("unexpected packet")]
    Illegal,
}

pub trait PacketReadExt {
    fn read(&mut self) -> impl Future<Output = Result<Packet, Error>>;
}

pub trait PacketWriteExt {
    fn write(&mut self, packet: &Packet) -> impl Future<Output = Result<(), Error>>;
}

impl PacketReadExt for FramedRead<OwnedReadHalf, LengthDelimitedCodec> {
    async fn read(&mut self) -> Result<Packet, Error> {
        if let Some(bytes) = self.try_next().await? {
            let packet: Packet = rmp_serde::from_slice(&bytes)?;
            Ok(packet)
        } else {
            Err(Error::Close)
        }
    }
}

impl PacketWriteExt for FramedWrite<OwnedWriteHalf, LengthDelimitedCodec> {
    async fn write(&mut self, packet: &Packet) -> Result<(), Error> {
        let bytes = rmp_serde::to_vec(packet)?;
        eprintln!("sending: {bytes:x?}");
        self.send(bytes.into()).await?;

        Ok(())
    }
}

pub trait FramedSplitExt {
    fn framed_split(
        self,
    ) -> (
        FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
        FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>,
    );
}

impl FramedSplitExt for TcpStream {
    fn framed_split(
        self,
    ) -> (
        FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
        FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>,
    ) {
        let (reader, writer) = self.into_split();
        let codec = LengthDelimitedCodec::new();

        (
            FramedRead::new(reader, codec.clone()),
            FramedWrite::new(writer, codec),
        )
    }
}
