mod connection;
mod protocol;

use tokio::net::TcpListener;
use log::info;
use env_logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init(); // Initialize logger

    let listener = TcpListener::bind("127.0.0.1:5672").await?;
    info!("AMQP service listening on 127.0.0.1:5672");

    loop {
        let (socket, addr) = listener.accept().await?;
        info!("New connection from {:?}", addr);

        tokio::spawn(async move {
            if let Err(e) = connection::handle_connection(socket).await {
                eprintln!("Error handling connection from {:?}: {:?}", addr, e);
            }
        });
    }
}
