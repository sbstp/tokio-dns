use tokio_core::{LoopHandle, TcpStream, TcpListener, UdpSocket};
use tokio_core::io::IoFuture;

use super::common::{tcp_connect_seq, tcp_connect_par, tcp_listen_seq, udp_bind_seq};
use super::{CpuPoolResolver, ToEndpoint};

lazy_static! {
    static ref POOL: CpuPoolResolver = CpuPoolResolver::new(5);
}

/// An extension trait to add name resolution to objects.
pub trait DnsSupport {
    /// Create a new TcpStream connected to the specified endpoint.
    ///
    /// If the endpoint is a hostname, it will be resolved and every
    /// address returned will be tried in parallel.
    fn tcp_connect_par<'a, T>(&self, ep: T) -> IoFuture<TcpStream>
        where T: ToEndpoint<'a>;

    /// Create a new TcpStream connected to the specified endpoint.
    ///
    /// If the endpoint is a hostname, it will be resolved and every
    /// address returned will be tried one after the other.
    fn tcp_connect_seq<'a, T>(&self, ep: T) -> IoFuture<TcpStream>
        where T: ToEndpoint<'a>;

    /// Create a new TcpListener bound to the specified endpoint.
    ///
    /// If the endpoint is a hostname, it will be resolved and every
    /// address returned will be tried one after the other.
    fn tcp_listen_seq<'a, T>(&self, ep: T) -> IoFuture<TcpListener>
        where T: ToEndpoint<'a>;

    /// Create a new UdpSocket bound to the specified endpoint.
    ///
    /// If the endpoint is a hostname, it will be resolved and every
    /// address returned will be tried one after the other.
    fn udp_bind_seq<'a, T>(&self, ep: T) -> IoFuture<UdpSocket>
        where T: ToEndpoint<'a>;
}

impl DnsSupport for LoopHandle {
    fn tcp_connect_par<'a, T>(&self, ep: T) -> IoFuture<TcpStream>
        where T: ToEndpoint<'a>
    {
        tcp_connect_par(self.clone(), POOL.clone(), ep)
    }

    fn tcp_connect_seq<'a, T>(&self, ep: T) -> IoFuture<TcpStream>
        where T: ToEndpoint<'a>
    {
        tcp_connect_seq(self.clone(), POOL.clone(), ep)
    }

    fn tcp_listen_seq<'a, T>(&self, ep: T) -> IoFuture<TcpListener>
        where T: ToEndpoint<'a>
    {
        tcp_listen_seq(self.clone(), POOL.clone(), ep)
    }

    fn udp_bind_seq<'a, T>(&self, ep: T) -> IoFuture<UdpSocket>
        where T: ToEndpoint<'a>
    {
        udp_bind_seq(self.clone(), POOL.clone(), ep)
    }
}
