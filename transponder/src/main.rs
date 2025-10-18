use crate::{serial::Serial, tcp::TCP};
use tokio::{io, select, sync::mpsc};
use tokio_serial::SerialStream;

pub mod serial;
pub mod tcp;

pub enum Message {
    Serial(io::WriteHalf<SerialStream>),
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    let (tx, rx) = mpsc::channel(8);

    let (tcp, writer) = TCP::init(rx).await?;
    let serial = Serial::new(writer, tx).await;

    let serial_handle = tokio::spawn(serial.listen());
    let tcp_handle = tokio::spawn(tcp.listen());

    select! {
        result = serial_handle => {
            eprintln!("transponder: serial quit: {result:?}");
        },
        result = tcp_handle => {
            eprintln!("transponder: tcp quit: {result:?}");
        }
    }

    Ok(())
}
