# tokio-dns
Asynchronous name resolution utilities for the `futures` and `tokio-core` crates. Provides a basic resolver as well as extension traits for the `tokio-core` event loop. Look at the crate-level documentation for more details.

[![BuildStatus](https://api.travis-ci.org/sbstp/tokio-dns.svg?branch=master)](https://travis-ci.org/sbstp/tokio-dns)

[Documentation](https://sbstp.github.io/tokio-dns/tokio_dns/index.html)

## Demo
```rust
// Taken from examples/support.rs

// connect using the built-in resolver.
let co = lp.handle().tcp_connect_seq("rust-lang.org:80").and_then(|sock| {
    println!("conncted to {}", sock.peer_addr().unwrap());
    Ok(())
});
```

## License
[MIT](LICENSE-MIT) or [Apache](LICENSE-APACHE)
