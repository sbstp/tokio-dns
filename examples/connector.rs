extern crate futures;
extern crate tokio_core;
extern crate tokio_dns;

use futures::Future;
use tokio_core::Loop;
use tokio_dns::{Connector, CpuPoolResolver, DnsSupport};

fn main() {
    let mut lp = Loop::new().unwrap();

    let connector = Connector::new(lp.handle(), CpuPoolResolver::new(10));

    // connect using the connector's resolver.
    let fut = connector.tcp_connect_par("rust-lang.org:80").and_then(|sock| {
        println!("conncted to {}", sock.peer_addr().unwrap());
        Ok(())
    });

    lp.run(fut).unwrap();
}
