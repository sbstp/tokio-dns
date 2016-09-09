use std::io;
use std::net::{IpAddr, SocketAddr};

use futures::{self, Future};
use tokio_core::net::{TcpListener, TcpStream, UdpSocket};
use tokio_core::reactor::Handle;

use super::resolver::{CpuPoolResolver, Resolver};
use super::{Endpoint, ToEndpoint};

lazy_static! {
    static ref POOL: CpuPoolResolver = CpuPoolResolver::new(5);
}

pub type IoFuture<I> = Box<Future<Item=I, Error=io::Error>>;

/// Connect to the endpoint using the default resolver.
pub fn tcp_connect<'a, T>(ep: T, handle: &Handle) -> IoFuture<TcpStream>
    where T: ToEndpoint<'a>
{
    tcp_connect_with(ep, handle, POOL.clone())
}

/// Connect to the endpoint using a custom resolver.
pub fn tcp_connect_with<'a, T, R>(ep: T, handle: &Handle, resolver: R) -> IoFuture<TcpStream>
    where T: ToEndpoint<'a>, R: Resolver
{
    if_host_resolve(handle, ep, resolver, |handle, port, ip_addrs| {
        let mut prev: Option<IoFuture<TcpStream>> = None;

        // This loop chains futures one after another so they each try
        // to connect to an address in a sequential way.
        for ip_addr in ip_addrs {
            let addr = SocketAddr::new(ip_addr, port);
            let handle = handle.clone();

            prev = Some(match prev.take() {
                None => Box::new(TcpStream::connect(&addr, &handle)),
                Some(prev) => Box::new(prev.or_else(move |_| {
                    let addr = addr.clone();
                    Box::new(TcpStream::connect(&addr, &handle))
                })),
            });
        }

        // If this Option is None, it means that there were no addresses in the list.
        match prev.take() {
            Some(fut) => fut,
            None => Box::new(futures::failed(io::Error::new(io::ErrorKind::Other, "resolve returned no addresses"))),
        }
    }, |handle, addr| Box::new(TcpStream::connect(addr, &handle)))
}

/// Bind to the endpoint using the default resolver.
pub fn tcp_bind<'a, T>(ep: T, handle: &Handle) -> IoFuture<TcpListener>
    where T: ToEndpoint<'a>
{
    tcp_bind_with(ep, handle, POOL.clone())
}

/// Bind to the endpoint using a custom resolver.
pub fn tcp_bind_with<'a, T, R>(ep: T, handle: &Handle, resolver: R) -> IoFuture<TcpListener>
    where T: ToEndpoint<'a>, R: Resolver
{
    if_host_resolve(handle, ep, resolver, |handle, port, ip_addrs| {
        let mut last_err = None;

        for ip_addr in ip_addrs {
            let addr = SocketAddr::new(ip_addr, port);
            match TcpListener::bind(&addr, &handle) {
                Ok(sock) => return Box::new(futures::finished(sock)),
                Err(err) => last_err = Some(err),
            }
        }

        Box::new(futures::failed(last_err.unwrap_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "could not resolve to any address")
        })))
    }, |handle, addr| Box::new(futures::done(TcpListener::bind(&addr, &handle))))
}

/// Bind to the endpoint using the default resolver.
pub fn udp_bind<'a, T>(ep: T, handle: &Handle) -> IoFuture<UdpSocket>
    where T: ToEndpoint<'a>
{
    udp_bind_with(ep, handle, POOL.clone())
}

/// Bind to the endpoint using a custom resolver.
pub fn udp_bind_with<'a, T, R>(ep: T, handle: &Handle, resolver: R) -> IoFuture<UdpSocket>
    where T: ToEndpoint<'a>, R: Resolver
{
    if_host_resolve(handle, ep, resolver, |handle, port, ip_addrs| {
        let mut last_err = None;

        for ip_addr in ip_addrs {
            let addr = SocketAddr::new(ip_addr, port);
            match UdpSocket::bind(&addr, &handle) {
                Ok(sock) => return Box::new(futures::finished(sock)),
                Err(err) => last_err = Some(err),
            }
        }

        Box::new(futures::failed(last_err.unwrap_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "could not resolve to any address")
        })))
    }, |handle, addr| Box::new(futures::done(UdpSocket::bind(&addr, &handle))))
}


// abstraction of the code that is common to all the functions
fn if_host_resolve<'a, T, R, F, E, S>(handle: &Handle, ep: T, resolver: R, func: F, elsef: E) -> IoFuture<S>
        where R: Resolver,
              T: ToEndpoint<'a>,
              F: FnOnce(Handle, u16, Vec<IpAddr>) -> IoFuture<S> + 'static,
              E: FnOnce(Handle, &SocketAddr) -> IoFuture<S>,
              S: 'static,
{
    let ep = match ep.to_endpoint() {
        Ok(ep) => ep,
        Err(e) => return Box::new(futures::failed(e)),
    };

    match ep {
        Endpoint::Host(host, port) => {
            let handle = handle.clone();
            Box::new(resolver.resolve(host).and_then(move |addrs| {
                func(handle, port, addrs)
            }))
        }
        Endpoint::SocketAddr(ref addr) => {
            elsef(handle.clone(), addr)
        }
    }
}
