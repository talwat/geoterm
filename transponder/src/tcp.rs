use shared::{BufferedSplitExt, LOCALHOST, Packet, PacketReadExt, serializers::Serialize};
use tokio::{
    net::TcpStream,
    select,
    sync::mpsc::{self, Receiver},
};

use crate::Message;

pub struct TCP {
    pub(crate) reader: shared::Reader,
    pub(crate) rx: Receiver<Message>,
}

impl TCP {
    pub async fn listen(mut self) -> eyre::Result<()> {
        let mut writer = None;

        loop {
            while writer.is_none() {
                let message = self.rx.recv().await.unwrap();
                match message {
                    Message::Serial(serial) => writer = Some(serial),
                }
            }

            let packet = select! {
                Some(Message::Serial(serial)) = self.rx.recv() => {
                    writer = Some(serial);
                    continue;
                },
                packet = self.reader.read_packet() => packet?,
            };

            if matches!(&packet, Packet::Round { .. }) {
                eprintln!("transponder: round packet");
            } else {
                eprintln!("transponder: client-bound tcp packet: {packet:?}");
            }

            let Err(error) = packet.serialize(&mut writer.as_mut().unwrap()).await else {
                continue;
            };

            eprintln!("transponder: error serializing: {error:?}");
            writer = None;
        }
    }

    pub async fn init(rx: mpsc::Receiver<Message>) -> eyre::Result<(Self, shared::Writer)> {
        let tcp = TcpStream::connect(LOCALHOST).await?;
        let (reader, writer) = tcp.buffered_split();

        Ok((Self { reader, rx }, writer))
    }
}
