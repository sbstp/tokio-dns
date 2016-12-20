# tokio-dns
Asynchronous name resolution utilities for the `futures` and `tokio-core` crates. Look at the crate-level documentation for more details.

[![BuildStatus](https://api.travis-ci.org/sbstp/tokio-dns.svg?branch=master)](https://travis-ci.org/sbstp/tokio-dns)

[Documentation](https://docs.rs/tokio-dns-unofficial)

This library [has been packaged to crates.io](https://crates.io/crates/tokio-dns-unofficial). Note that its name on crates.io is `tokio-dns-unofficial`, but the crate's name is `tokio-dns` (when using `extern crate ...`).

## Demo
```rust
// Taken from examples/basic.rs

// connect using the built-in resolver.
let conn = tcp_connect("rust-lang.org:80", &lp.handle()).and_then(|sock| {
    println!("conncted to {}", sock.peer_addr().unwrap());
    Ok(())
});
```

## License
[MIT](LICENSE-MIT) or [Apache](LICENSE-APACHE)
