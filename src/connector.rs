use tokio_core::{LoopHandle, TcpStream};
use tokio_core::io::IoFuture;

use super::{DnsSupport, Resolver, ToEndpoint};
use super::common::{tcp_connect_par, tcp_connect_seq};

/// A helper for creating connections.
///
/// This object is a wrapper around a `LoopHandle` and a resolver.
/// It helps initiate connections using endpoints, and offers
/// bultin address translation.
pub struct Connector<R> where R: Clone + Resolver {
    handle: LoopHandle,
    resolver: R,
}

impl<R> Connector<R> where R: Clone + Resolver {
    /// Create a new `Connector`.
    ///
    /// The `handle` can be obtained using the `handle()` method of the `Loop` object.
    /// The `resolver_threads` parameter is the amount of threads given to the resolver's
    /// thread pool. The `mode` parameter tells the connector how to attempt connection
    /// when the resolver yields more than one IP address.
    pub fn new(handle: LoopHandle, resolver: R) -> Self {
        Connector {
            handle: handle,
            resolver: resolver,
        }
    }
}

impl<R> DnsSupport for Connector<R> where R: Clone + Resolver {
    fn tcp_connect_par<'a, T>(&self, ep: T) -> IoFuture<TcpStream>
        where T: ToEndpoint<'a>
    {
        tcp_connect_par(self.handle.clone(), self.resolver.clone(), ep)
    }

    fn tcp_connect_seq<'a, T>(&self, ep: T) -> IoFuture<TcpStream>
        where T: ToEndpoint<'a>
    {
        tcp_connect_seq(self.handle.clone(), self.resolver.clone(), ep)
    }
}
