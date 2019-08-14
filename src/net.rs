use std::io;
use std::net::{IpAddr, SocketAddr};

use futures::{compat::*, prelude::*};
use tokio::net;

use crate::endpoint::{Endpoint, ToEndpoint};
use crate::resolver::{CpuPoolResolver, Resolver};

lazy_static::lazy_static! {
    static ref POOL: CpuPoolResolver = CpuPoolResolver::new(5);
}

/// Resolve a hostname to a sequence of ip addresses using the default resolver.
///
/// # Example
/// ```
/// tokio_dns::resolve_ip_addr("rust-lang.org");
/// ```
pub async fn resolve_ip_addr(host: &str) -> io::Result<Vec<IpAddr>> {
    POOL.resolve(host).await
}

/// Resolve a hostname to a sequence of ip addresses using a custom resolver.
///
/// # Example
/// ```
/// # use tokio_dns::CpuPoolResolver;
/// let resolver = CpuPoolResolver::new(10);
///
/// tokio_dns::resolve_ip_addr_with("rust-lang.org", resolver.clone());
/// ```
pub async fn resolve_ip_addr_with<R>(host: &str, resolver: R) -> io::Result<Vec<IpAddr>>
where
    R: Resolver,
{
    resolver.resolve(host).await
}

/// Resolve an endpoint to a sequence of socket addresses using the default resolver.
///
/// # Example
/// ```
/// tokio_dns::resolve_sock_addr(("rust-lang.org", 80));
/// ```
pub async fn resolve_sock_addr<'a, T>(endpoint: T) -> io::Result<Vec<SocketAddr>>
where
    T: ToEndpoint<'a>,
{
    resolve_endpoint(endpoint, POOL.clone()).await
}

/// Resolve an endpoint to a sequence of socket addresses using a custom resolver.
///
/// # Example
/// ```
/// # use tokio_dns::CpuPoolResolver;
/// let resolver = CpuPoolResolver::new(10);
///
/// tokio_dns::resolve_sock_addr_with(("rust-lang.org", 80), resolver.clone());
/// ```
pub async fn resolve_sock_addr_with<'a, T, R>(
    endpoint: T,
    resolver: R,
) -> io::Result<Vec<SocketAddr>>
where
    T: ToEndpoint<'a>,
    R: Resolver,
{
    resolve_endpoint(endpoint, resolver).await
}

/// Shim for tokio::net::TcpStream
pub struct TcpStream;

impl TcpStream {
    /// Connect to the endpoint using the default resolver.
    pub async fn connect<'a, T>(ep: T) -> io::Result<net::TcpStream>
    where
        T: ToEndpoint<'a>,
    {
        TcpStream::connect_with(ep, POOL.clone()).await
    }

    /// Connect to the endpoint using a custom resolver.
    pub async fn connect_with<'a, T, R>(ep: T, resolver: R) -> io::Result<net::TcpStream>
    where
        T: ToEndpoint<'a>,
        R: Resolver,
    {
        try_until_ok(resolve_endpoint(ep, resolver).await?, move |addr| {
            net::TcpStream::connect(&addr).compat().boxed()
        })
        .await
    }
}

/// Shim for tokio::net::TcpListener
pub struct TcpListener;

impl TcpListener {
    /// Bind to the endpoint using the default resolver.
    pub async fn bind<'a, T>(ep: T) -> io::Result<net::TcpListener>
    where
        T: ToEndpoint<'a>,
    {
        TcpListener::bind_with(ep, POOL.clone()).await
    }

    /// Bind to the endpoint using a custom resolver.
    pub async fn bind_with<'a, T, R>(ep: T, resolver: R) -> io::Result<net::TcpListener>
    where
        T: ToEndpoint<'a>,
        R: Resolver,
    {
        try_until_ok(resolve_endpoint(ep, resolver).await?, move |addr| {
            future::ready(net::TcpListener::bind(&addr))
        })
        .await
    }
}

/// Shim for tokio::net::UdpSocket
pub struct UdpSocket;

impl UdpSocket {
    /// Bind to the endpoint using the default resolver.
    pub async fn bind<'a, T>(ep: T) -> io::Result<net::UdpSocket>
    where
        T: ToEndpoint<'a>,
    {
        UdpSocket::bind_with(ep, POOL.clone()).await
    }

    /// Bind to the endpoint using a custom resolver.
    pub async fn bind_with<'a, T, R>(ep: T, resolver: R) -> io::Result<net::UdpSocket>
    where
        T: ToEndpoint<'a>,
        R: Resolver,
    {
        try_until_ok(resolve_endpoint(ep, resolver).await?, move |addr| {
            future::ready(net::UdpSocket::bind(&addr))
        })
        .await
    }
}

/// Resolves endpoint into a vector of socket addresses.
async fn resolve_endpoint<'a, T, R>(ep: T, resolver: R) -> io::Result<Vec<SocketAddr>>
where
    R: Resolver,
    T: ToEndpoint<'a>,
{
    let ep = match ep.to_endpoint() {
        Ok(ep) => ep,
        Err(e) => return Err(e),
    };
    Ok({
        match ep {
            Endpoint::Host(host, port) => resolver
                .resolve(host)
                .await?
                .into_iter()
                .map(|addr| SocketAddr::new(addr, port))
                .collect(),
            Endpoint::SocketAddr(addr) => vec![addr],
        }
    })
}

async fn try_until_ok<F, R, I>(addrs: Vec<SocketAddr>, f: F) -> io::Result<I>
where
    F: Fn(SocketAddr) -> R + Send + 'static,
    R: Future<Output = io::Result<I>> + Send + 'static,
    I: Send + 'static,
{
    for addr in addrs {
        if let Ok(i) = f(addr).await {
            return Ok(i);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        "could not resolve to any address",
    ))
}
