use crate::{serial::Serial, tcp::TCP};
use tokio::select;

pub mod serial;
pub mod tcp;

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    let tcp = TCP::init().await?;
    let serial = Serial::init().await?;

    let serial_handle = tokio::spawn(Serial::listen(serial.reader, tcp.writer));
    let tcp_handle = tokio::spawn(TCP::listen(tcp.reader, serial.writer));

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
