//! This crate offers tools for asynchronous name resolution, and extensions to
//! the `tokio_core` crate.
//!
//! First, `Endpoint` and `ToEndpoint` behave very much like `SocketAddr` and
//! `ToSocketAddrs` from the standard library. The main difference is that the
//! `ToEndpoint` trait does not perform any name resolution. If simply detect
//! whether the given endpoint is a socket address or a host name. Then, it
//! is up to a resolver to perform name resolution.
//!
//! The `Resolver` trait describes an abstract, asynchronous resolver. This crate
//! provides one (for now) implementation of a resolver, the `CpuPoolResolver`.
//! It uses a thread pool and the `ToSocketAddrs` trait to perform name resolution.
//!
//! The crate level functions `tcp_connect`, `tcp_listen` and `udp_bind` support
//! name resolution via a lazy static `CpuPoolResolver` using 5 threads. Their
//!`*_with` counterpart take a resolver as an argument.
//!
//! [Git Repository](https://github.com/sbstp/tokio-dns)
#![deny(missing_docs)]

extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod endpoint;
mod net;
mod resolver;

pub use endpoint::{Endpoint, ToEndpoint};
pub use net::{tcp_connect, tcp_connect_with, tcp_bind, tcp_bind_with, udp_bind, udp_bind_with};
pub use resolver::{CpuPoolResolver, Resolver};
