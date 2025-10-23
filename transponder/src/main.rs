use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

use crate::{serial::Serial, tcp::TCP};
use clap::Parser;
use shared::PORT;
use tokio::{io, select, sync::mpsc};
use tokio_serial::SerialStream;

pub mod serial;
pub mod tcp;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,
}

pub enum Message {
    Serial(io::WriteHalf<SerialStream>),
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let (tx, rx) = mpsc::channel(8);

    let address = SocketAddrV4::new(Ipv4Addr::from_str(&args.address)?, PORT);
    eprintln!("transponder: attempting to connect to {address}...");
    let (tcp, writer) = TCP::init(rx, address.clone()).await?;
    let serial = Serial::new(writer, tx).await;

    let serial_handle = tokio::spawn(serial.listen());
    let tcp_handle = tokio::spawn(tcp.listen(address));

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
