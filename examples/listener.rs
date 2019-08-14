use futures::prelude::*;
use tokio_dns::TcpListener;

#[tokio::main]
async fn main() {
    // connect using the built-in resolver.
    match TcpListener::bind("localhost:3000").await {
        Ok(listener) => {
            println!("Ready to accept");
            let mut s = listener.incoming();
            while let Some(sock) = s.next().await {
                println!("Accepted connection from {:?}", sock.unwrap().peer_addr());
            }
        }
        Err(err) => println!("Error binding socket {:?}", err),
    }
}
