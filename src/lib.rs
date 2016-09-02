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
//! The `DnsSupport` trait is an extension trait that adds name resolution to an
//! object. There's an implementation for `LoopHandle` that uses a `lazy_static!`
//! `CpuPoolResolver` of 5 threads. This is the easiest way of using this crate.
//! Refer to the `examples/support.rs` file to see an example of it in use.
//!
//! This crate also offers a `Connector` struct, that is more customizable.
//! The `DnsSupport` trait is implemented for it and it will use the resolver
//! you give it when resolving. Refer to the `examples/connector.rs` file to
//! see an example of it.
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

mod common;
mod connector;
mod endpoint;
mod select_all_ok;
mod support;

use std::io;
use std::net::{IpAddr, ToSocketAddrs};

use futures::{BoxFuture, Future};
use futures_cpupool::CpuPool;
use tokio_core::io::IoFuture;

pub use connector::Connector;
pub use endpoint::{Endpoint, ToEndpoint};
pub use support::DnsSupport;

/// The Resolver trait represents an object capable of
/// resolving host names into IP addresses.
pub trait Resolver {
    /// Given a host name, this function returns a Future which
    /// will eventually resolve into a list of IP addresses.
    fn resolve(&self, host: &str) -> BoxFuture<Vec<IpAddr>, io::Error>;
}

/// A resolver based on a thread pool.
///
/// This resolver uses the `ToSocketAddrs` trait inside
/// a thread to provide non-blocking address resolving.
#[derive(Clone)]
pub struct CpuPoolResolver {
    pool: CpuPool,
}

impl CpuPoolResolver {
    /// Create a new CpuPoolResolver with the given number of threads.
    pub fn new(num_threads: usize) -> Self {
        CpuPoolResolver {
            pool: CpuPool::new(num_threads),
        }
    }
}

impl Resolver for CpuPoolResolver {
    fn resolve(&self, host: &str) -> IoFuture<Vec<IpAddr>> {
        let host = format!("{}:0", host);
        self.pool.spawn_fn(move || {
            match host[..].to_socket_addrs() {
                Ok(it) => Ok(it.map(|s| s.ip()).collect()),
                Err(e) => Err(e),
            }
        }).boxed()
    }
}
