extern crate futures;
extern crate tokio_core;
extern crate tokio_dns;

use futures::Future;
use futures::stream::Stream;
use tokio_core::reactor::Core;
use tokio_dns::tcp_bind;

fn main() {
    let mut lp = Core::new().unwrap();

    // connect using the built-in resolver.
    let serv = tcp_bind("localhost:3000", lp.remote()).and_then(|listener| {
        println!("ready to accept");
        listener.incoming().for_each(|(_, addr)| {
            println!("accepted connection from {:?}", addr);
            Ok(())
        })
    });

    lp.run(serv).unwrap();
}
