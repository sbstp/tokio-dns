extern crate futures;
extern crate tokio_core;
extern crate tokio_dns;

use futures::Future;
use tokio_core::Loop;
use tokio_dns::DnsSupport;

fn main() {
    let mut lp = Loop::new().unwrap();

    // connect using the built-in resolver.
    let co = lp.handle().tcp_connect_seq("rust-lang.org:80").and_then(|sock| {
        println!("conncted to {}", sock.peer_addr().unwrap());
        Ok(())
    });

    lp.run(co).unwrap();
}
