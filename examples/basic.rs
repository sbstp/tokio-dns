extern crate futures;
extern crate tokio;
extern crate tokio_dns;

use futures::Future;
use tokio_dns::TcpStream;

fn main() {
    // connect using the built-in resolver.
    let connector = TcpStream::connect("rust-lang.org:80")
        .and_then(|sock| {
            println!("Connected to {}", sock.peer_addr().unwrap());
            Ok(())
        })
        .then(|_| Ok(()));

    tokio::run(connector);
}
