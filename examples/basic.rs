use futures::prelude::*;
use tokio_dns::TcpStream;

fn main() {
    tokio::run(
        async move {
            // connect using the built-in resolver.
            match TcpStream::connect("rust-lang.org:80").await {
                Ok(sock) => println!("Connected to {}", sock.peer_addr().unwrap()),
                Err(err) => println!("Error connecting {:?}", err),
            }
        }
            .unit_error()
            .boxed()
            .compat(),
    );
}
