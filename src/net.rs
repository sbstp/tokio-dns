use std::io;
use std::net::SocketAddr;

use futures::{self, Future, IntoFuture};
use futures::stream::Stream;
use tokio_core::net::{TcpListener, TcpStream, UdpSocket};
use tokio_core::reactor::{Handle, Remote};
use tokio_io::IoFuture;

use super::resolver::{CpuPoolResolver, Resolver};
use super::{Endpoint, ToEndpoint};
use boxed;

lazy_static! {
    static ref POOL: CpuPoolResolver = CpuPoolResolver::new(5);
}

/// Connect to the endpoint using the default resolver.
pub fn tcp_connect<'a, T>(ep: T, handle: Remote) -> IoFuture<TcpStream>
where
    T: ToEndpoint<'a>,
{
    tcp_connect_with(ep, handle, POOL.clone())
}

/// Connect to the endpoint using a custom resolver.
pub fn tcp_connect_with<'a, T, R>(ep: T, remote: Remote, resolver: R) -> IoFuture<TcpStream>
where
    T: ToEndpoint<'a>,
    R: Resolver,
{
    boxed(resolve_endpoint(ep, resolver).and_then(move |addrs| {
        try_until_ok(addrs, move |addr| {
            with_handle(&remote, move |handle| TcpStream::connect(&addr, handle))
        })
    }))
}

/// Bind to the endpoint using the default resolver.
pub fn tcp_bind<'a, T>(ep: T, remote: Remote) -> IoFuture<TcpListener>
where
    T: ToEndpoint<'a>,
{
    tcp_bind_with(ep, remote, POOL.clone())
}

/// Bind to the endpoint using a custom resolver.
pub fn tcp_bind_with<'a, T, R>(ep: T, remote: Remote, resolver: R) -> IoFuture<TcpListener>
where
    T: ToEndpoint<'a>,
    R: Resolver,
{
    boxed(resolve_endpoint(ep, resolver).and_then(move |addrs| {
        try_until_ok(addrs, move |addr| {
            with_handle(&remote, move |handle| TcpListener::bind(&addr, handle))
        })
    }))
}

/// Bind to the endpoint using the default resolver.
pub fn udp_bind<'a, T>(ep: T, remote: Remote) -> IoFuture<UdpSocket>
where
    T: ToEndpoint<'a>,
{
    udp_bind_with(ep, remote, POOL.clone())
}

/// Bind to the endpoint using a custom resolver.
pub fn udp_bind_with<'a, T, R>(ep: T, remote: Remote, resolver: R) -> IoFuture<UdpSocket>
where
    T: ToEndpoint<'a>,
    R: Resolver,
{
    boxed(resolve_endpoint(ep, resolver).and_then(move |addrs| {
        try_until_ok(addrs, move |addr| {
            with_handle(&remote, move |handle| UdpSocket::bind(&addr, handle))
        })
    }))
}

/// Resolves endpoint into a vector of socket addresses.
fn resolve_endpoint<'a, T, R>(ep: T, resolver: R) -> IoFuture<Vec<SocketAddr>>
where
    R: Resolver,
    T: ToEndpoint<'a>,
{
    let ep = match ep.to_endpoint() {
        Ok(ep) => ep,
        Err(e) => return boxed(futures::failed(e)),
    };
    match ep {
        Endpoint::Host(host, port) => boxed(resolver.resolve(host).map(move |addrs| {
            addrs
                .into_iter()
                .map(|addr| SocketAddr::new(addr, port))
                .collect()
        })),
        Endpoint::SocketAddr(addr) => boxed(futures::finished(vec![addr])),
    }
}

fn try_until_ok<F, R, I>(addrs: Vec<SocketAddr>, f: F) -> IoFuture<I>
where
    F: Fn(SocketAddr) -> R + Send + 'static,
    R: IntoFuture<Item = I, Error = io::Error> + 'static,
    R::Future: Send + 'static,
    <R::Future as Future>::Error: From<io::Error>,
    I: Send + 'static,
{
    let result = Err(io::Error::new(
        io::ErrorKind::Other,
        "could not resolve to any address",
    ));
    boxed(
        futures::stream::iter_ok(addrs.into_iter())
            .fold(result, move |prev, addr| {
                match prev {
                    Ok(i) => {
                        // Keep first successful result.
                        boxed(futures::finished(Ok(i)))
                    }
                    Err(..) => {
                        // Ignore previous error and try next address.
                        let future = f(addr).into_future();
                        // Lift future error into item to avoid short-circuit exit from fold.
                        boxed(future.then(Ok))
                    }
                }
            })
            .and_then(|r| r),
    )
}

/// Invokes functor with event loop handle obtained from a remote.
fn with_handle<F, R, I>(remote: &Remote, f: F) -> IoFuture<I>
where
    F: FnOnce(&Handle) -> R + Send + 'static,
    R: IntoFuture<Item = I, Error = io::Error> + Send + 'static,
    R::Future: Send + 'static,
{
    let (tx, rx) = futures::oneshot();
    remote.spawn(move |handle| {
        let _ = tx.send(f(handle));
        Ok(())
    });
    boxed(rx.then(|r| r.expect("canceled")))
}
