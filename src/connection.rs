use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::{info, warn};
use crate::protocol::{parse_amqp_header, parse_amqp_frame};

pub async fn handle_connection(mut socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut header_buf = [0u8; 8];
    socket.read_exact(&mut header_buf).await?;
    match parse_amqp_header(&header_buf) {
        Ok(_) => info!("AMQP header received and validated"),
        Err(e) => {
            warn!("Invalid AMQP header: {:?}", e);
            // You might want to drop the connection or send an error
            socket.write_all(b"Invalid AMQP header").await?;
            return Err(e.into());
        }
    }

    let mut buf = vec![0u8; 4096];
    loop {
        let n = match socket.read(&mut buf).await {
            Ok(0) => {
                // EOF - connection closed by client
                info!("Connection closed by client");
                return Ok(());
            }
            Ok(n) => n,
            Err(e) => {
                warn!("Failed to read from socket: {:?}", e);
                return Err(e.into());
            }
        };

        let incoming = &buf[..n];
        match parse_amqp_frame(incoming) {
            Ok(frame) => {
                info!("Received frame: {:?}", frame);
                socket.write_all(b"Frame received").await?;
            }
            Err(e) => {
                warn!("Frame parse error: {:?}", e);
                socket.write_all(b"Invalid frame").await?;
            }
        }
    }
}
