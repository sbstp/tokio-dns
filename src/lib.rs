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
#![warn(missing_docs)]

mod endpoint;
mod net;
mod resolver;

use std::{io, pin::Pin};

use futures::prelude::*;

pub use crate::endpoint::{Endpoint, ToEndpoint};
#[allow(deprecated)]
pub use crate::net::{
    resolve_ip_addr, resolve_ip_addr_with, resolve_sock_addr, resolve_sock_addr_with, TcpListener,
    TcpStream, UdpSocket,
};
pub use crate::resolver::{CpuPoolResolver, Resolver};
