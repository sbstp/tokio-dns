use futures::prelude::*;
use tokio_dns::CpuPoolResolver;

fn main() {
    // create a custom, 10 thread CpuPoolResolver
    let resolver = CpuPoolResolver::new(10);

    tokio::run({
        // resolver is moved into the function, cloning it allows it to be used again after this call
        let resolver = resolver.clone();
        async move {
            match tokio_dns::resolve_sock_addr_with("rust-lang.org:80", resolver.clone()).await {
                Ok(addrs) => println!("Socket addresses {:#?}", addrs),
                Err(err) => println!("Error resolve address {:?}", err),
            }
        }
            .unit_error()
            .boxed()
            .compat()
    });
}
