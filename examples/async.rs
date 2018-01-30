extern crate futures;
extern crate tokio_core;
extern crate tokio_dns;

use futures::Future;
use tokio_core::reactor::Core;
use tokio_dns::tcp_connect_with;


fn main() {
    let mut lp = Core::new().unwrap();

    let async_dns = tokio_dns::AsyncResolver::system_config(lp.handle())
        .expect("async resolver initialized");

    // connect using the built-in resolver.
    let conn = tcp_connect_with("rust-lang.org:80",
            &lp.handle(), async_dns.clone())
        .and_then(|sock| {
            println!("connected to {}", sock.peer_addr().unwrap());
            Ok(())
        });

    lp.run(conn).unwrap();
}
