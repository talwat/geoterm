use std::time::Duration;

use shared::{Packet, PacketWriteExt, deserializers::Deserialize};
use tokio::{io::ReadHalf, sync::mpsc::Sender, time::sleep};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

use crate::Message;

pub struct Serial {
    pub(crate) writer: shared::Writer,
    pub(crate) tx: Sender<Message>,
}

impl Serial {
    pub async fn connect(&mut self, mut reader: ReadHalf<SerialStream>) -> eyre::Result<()> {
        loop {
            match Packet::deserialize(&mut reader).await {
                Ok(packet) => {
                    eprintln!("transponder(serial): server-bound serial packet: {packet:?}");
                    self.writer.write_packet(packet).await?;
                }
                Err(error) => {
                    eprintln!(
                        "transponder(serial): error parsing server-bound serial packet: {error:?}"
                    );
                    // TODO: Don't just quit like this, weakling.
                    break Ok(());
                }
            }
        }
    }

    pub async fn listen(mut self) -> eyre::Result<()> {
        loop {
            let files: Vec<_> = std::fs::read_dir("/dev")
                .unwrap()
                .filter_map(|e| e.ok())
                .filter_map(|e| e.path().to_str().map(|s| s.to_string()))
                .filter(|p| p.starts_with("/dev/cu.usbmodem"))
                .collect();

            if files.is_empty() {
                sleep(Duration::from_millis(50)).await;
                continue;
            }

            let path = &files[0];
            let Ok((reader, writer)) = tokio_serial::new(path, 9600 * 4)
                .open_native_async()
                .and_then(|x| Ok(tokio::io::split(x)))
            else {
                continue;
            };
            eprintln!("transponder(serial): new serial connection: {path}");
            self.tx.send(Message::Serial(writer)).await?;

            let result = self.connect(reader).await;
            self.writer.write_packet(Packet::SoftQuit).await?;
            eprintln!("transponder(serial): lost serial connection");
            if let Err(error) = result {
                eprintln!("-> {error:?}");
            }
        }
    }

    pub async fn new(writer: shared::Writer, tx: Sender<Message>) -> Self {
        Self { writer, tx }
    }
}
