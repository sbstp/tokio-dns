use tokio_dns::TcpStream;

#[tokio::main]
async fn main() {
    // connect using the built-in resolver.
    match TcpStream::connect("rust-lang.org:80").await {
        Ok(sock) => println!("Connected to {}", sock.peer_addr().unwrap()),
        Err(err) => println!("Error connecting {:?}", err),
    }
}
