use shared::{ClientOptions, FramedSplitExt, Packet, PacketReadExt, PacketWriteExt};
use tokio::{
    io::AsyncWriteExt,
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::mpsc,
    task::JoinHandle,
};
use tokio_util::codec::{self, FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::{Error, Message};

pub struct Client {
    pub id: usize,
    pub ready: bool,
    pub options: Option<ClientOptions>,
    writer: FramedWrite<OwnedWriteHalf, codec::LengthDelimitedCodec>,
    handle: JoinHandle<Result<(), Error>>,

    #[allow(dead_code)]
    tx: mpsc::Sender<Message>,
}

impl Client {
    async fn listener(
        id: usize,
        tx: mpsc::Sender<Message>,
        mut reader: FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
    ) -> Result<(), Error> {
        loop {
            let packet = reader.read().await;
            let is_err = packet.is_err();
            tx.send(Message::Packet(id, packet)).await?;

            if is_err {
                break;
            };
        }

        Ok(())
    }

    pub async fn write(&mut self, packet: &Packet) -> Result<(), Error> {
        self.writer.write(packet).await?;
        Ok(())
    }

    pub async fn new(
        id: usize,
        tx: mpsc::Sender<Message>,
        socket: TcpStream,
    ) -> Result<Self, Error> {
        let (reader, writer) = socket.framed_split();
        let handle = tokio::spawn(Self::listener(id, tx.clone(), reader));

        Ok(Self {
            handle,
            id,
            tx,
            options: None,
            ready: false,
            writer,
        })
    }

    pub async fn close(self) {
        self.handle.abort();
        let _ = self.writer.into_inner().shutdown().await;
        eprintln!("server (client {}): closed", self.id);
    }
}
