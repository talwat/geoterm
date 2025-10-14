use shared::{Packet, PacketWriteExt, deserializers::Deserialize};
use tokio::io;
use tokio_serial::{SerialPortBuilderExt, SerialStream};

pub struct Serial {
    pub(crate) reader: io::ReadHalf<SerialStream>,
    pub(crate) writer: io::WriteHalf<SerialStream>,
}

impl Serial {
    pub async fn listen(
        mut reader: io::ReadHalf<SerialStream>,
        mut writer: shared::Writer,
    ) -> eyre::Result<()> {
        loop {
            match Packet::deserialize(&mut reader).await {
                Ok(packet) => {
                    eprintln!("transponder: server-bound serial packet: {packet:?}");
                    writer.write_packet(packet).await?;
                }
                Err(error) => {
                    eprintln!("transponder: error parsing server-bound serial packet: {error:?}");
                    break Ok(());
                }
            }
        }
    }

    pub async fn init() -> eyre::Result<Self> {
        let serial =
            tokio_serial::new("/dev/cu.usbmodem13010131AE1", 9600 * 4).open_native_async()?;
        let (reader, writer) = tokio::io::split(serial);

        Ok(Self { reader, writer })
    }
}
