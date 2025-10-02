use std::{num::ParseFloatError, str::Utf8Error};

use tokio::{io, sync::mpsc::error::SendError};

use crate::Message;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("utf8 encoding/decoding failed")]
    Utf8(#[from] Utf8Error),

    #[error("sending internal message failed")]
    Send(#[from] SendError<Message>),

    #[error("io failure")]
    Io(#[from] io::Error),

    #[error("packet error")]
    Packet(#[from] shared::Error),

    #[error("invalid lat/long request data")]
    Float(#[from] ParseFloatError),

    #[error("network request failed")]
    Request(#[from] reqwest::Error),

    #[error("image editing failed")]
    Image(#[from] image::ImageError),
}
