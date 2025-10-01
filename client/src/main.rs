use shared::{ClientOptions, FramedSplitExt, Packet, PacketReadExt, PacketWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:3000").await?;
    let (mut reader, mut writer) = stream.framed_split();

    writer
        .write(&Packet::Init {
            options: ClientOptions {
                user: "bob".to_string(),
            },
        })
        .await?;

    let read = reader.read().await?;
    dbg!(read);

    loop {
        eprintln!("waiting...");
        let Ok(read) = reader.read().await else {
            break;
        };

        dbg!(read);
    }

    Ok(())
}
