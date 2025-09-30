use futures::{SinkExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
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
    Confirmed { id: usize, options: ClientOptions }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Socket closed")]
    Closed,

    #[error("IO failure")]
    Io(#[from] std::io::Error),

    #[error("Packet decoding failed")]
    Decode(#[from] rmp_serde::decode::Error),

    #[error("Packet encoding failed")]
    Encode(#[from] rmp_serde::encode::Error),

    #[error("Unexpected packet")]
    Illegal,

    #[error("Unknown error: {0}")]
    Unknown(#[from] eyre::Error),
}

impl Packet {
    pub async fn read(framed: &mut FramedRead<&mut OwnedReadHalf, LengthDelimitedCodec>) -> Result<Packet, Error> {
        if let Some(bytes) = framed.try_next().await? {
            let packet: Packet = rmp_serde::from_slice(&bytes)?;
            Ok(packet)
        } else {
            Err(Error::Closed)
        }
    }

    pub async fn write(writer: &mut OwnedWriteHalf, packet: &Packet) -> Result<(), Error> {
        let mut framed = FramedWrite::new(writer, LengthDelimitedCodec::new());
        let bytes = rmp_serde::to_vec(packet)?;

        framed.send(bytes.into()).await?;
        Ok(())
    }
}
