extern crate futures;
extern crate tokio_core;
extern crate tokio_dns;

use futures::Future;
use tokio_core::reactor::Core;
use tokio_dns::tcp_connect;

fn main() {
    let mut lp = Core::new().unwrap();

    // connect using the built-in resolver.
    let conn = tcp_connect("rust-lang.org:80", &lp.handle()).and_then(|sock| {
        println!("connected to {}", sock.peer_addr().unwrap());
        Ok(())
    });

    lp.run(conn).unwrap();
}
