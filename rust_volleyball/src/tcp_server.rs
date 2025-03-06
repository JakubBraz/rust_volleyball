use std::net::SocketAddr;
use tokio::io::AsyncReadExt;

pub fn start() {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap()
        .block_on(async { run().await });
    log::error!("TCP server stopped");
}

async fn run() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:12541").await.expect("Cannot bind");
    loop {
        log::debug!("Waiting for connections...");
        match listener.accept().await {
            Ok((stream, addr)) => {
                tokio::spawn(async move {
                    handle_connection(stream, addr).await;
                });
            }
            Err(e) => {
                log::error!("Could not accept connection: {}", e);
            }
        }
    }
}

async fn handle_connection(mut stream: tokio::net::TcpStream, addr: SocketAddr) {
    let mut buffer = vec![0; 1024];
    log::info!("TCP connection accepted: {:?}", addr);
    loop {
        log::debug!("Waiting for data...");
        match stream.read(&mut buffer).await {
            Ok(len) => {
                log::debug!("Read {} bytes from {} client: {:?}", len, addr, &buffer[..len]);
            }
            Err(e) => {
                log::warn!("Error reading from stream: {}", e);
                break;
            }
        }
    }
    log::info!("TCP connection closed");
}
