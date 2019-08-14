use futures::{compat::*, prelude::*};
use tokio_dns::TcpListener;

fn main() {
    tokio::run(
        async move {
            // connect using the built-in resolver.
            match TcpListener::bind("localhost:3000").await {
                Ok(listener) => {
                    println!("Ready to accept");
                    let mut s = listener.incoming().compat();
                    while let Some(sock) = s.next().await {
                        println!("Accepted connection from {:?}", sock.unwrap().peer_addr());
                    }
                }
                Err(err) => println!("Error binding socket {:?}", err),
            }
        }
            .unit_error()
            .boxed()
            .compat(),
    );
}
