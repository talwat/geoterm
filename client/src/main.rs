use shared::{ClientOptions, FramedSplitExt, Packet};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:3000").await?;
    let (mut reader, mut writer) = stream.framed_split();

    Packet::write(
        &mut writer,
        &Packet::Init {
            options: ClientOptions {
                user: "bob".to_string(),
            },
        },
    )
    .await?;

    let read = Packet::read(&mut reader).await?;
    dbg!(read);

    Ok(())
}
