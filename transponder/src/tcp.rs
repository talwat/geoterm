use shared::{BufferedSplitExt, LOCALHOST, PacketReadExt, serializers::Serialize};
use tokio::{io, net::TcpStream};
use tokio_serial::SerialStream;

pub struct TCP {
    pub(crate) reader: shared::Reader,
    pub(crate) writer: shared::Writer,
}

impl TCP {
    pub async fn listen(
        mut reader: shared::Reader,
        mut writer: io::WriteHalf<SerialStream>,
    ) -> eyre::Result<()> {
        while let Ok(packet) = reader.read_packet().await {
            eprintln!("transponder: client-bound tcp packet: {packet:?}");
            packet.serialize(&mut writer).await?;
        }

        Ok(())
    }

    pub async fn init() -> eyre::Result<Self> {
        let tcp = TcpStream::connect(LOCALHOST).await?;
        let (reader, writer) = tcp.buffered_split();

        Ok(Self { reader, writer })
    }
}
