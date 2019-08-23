use std::io;
use std::net::{IpAddr, ToSocketAddrs};
use std::pin::Pin;
use std::str;

use futures::prelude::*;

/// The Resolver trait represents an object capable of
/// resolving host names into IP addresses.
pub trait Resolver {
    /// Given a host name, this function returns a Future which
    /// will eventually produce a list of IP addresses.
    fn resolve(&self, host: &str) -> Pin<Box<dyn Future<Output = io::Result<Vec<IpAddr>>> + Send>>;
}

/// A resolver based on a thread pool.
///
/// This resolver uses the `ToSocketAddrs` trait inside
/// a thread to provide non-blocking address resolving.
#[derive(Clone)]
pub struct CpuPoolResolver {
    pool: futures::executor::ThreadPool,
}

impl CpuPoolResolver {
    /// Create a new CpuPoolResolver with the given number of threads.
    pub fn new(num_threads: usize) -> Self {
        CpuPoolResolver {
            pool: futures::executor::ThreadPool::builder()
                .pool_size(num_threads)
                .create()
                .unwrap(),
        }
    }
}

impl Resolver for CpuPoolResolver {
    fn resolve(&self, host: &str) -> Pin<Box<dyn Future<Output = io::Result<Vec<IpAddr>>> + Send>> {
        let host = format!("{}:0", host);
        let pool = self.pool.clone();

        async move {
            let (tx, rx) = futures::channel::oneshot::channel();

            pool.spawn_obj_ok(
                async move {
                    let _ = tx.send(match host[..].to_socket_addrs() {
                        Ok(it) => Ok(it.map(|s| s.ip()).collect()),
                        Err(e) => Err(e),
                    });
                }
                    .boxed()
                    .into(),
            );

            rx.await.unwrap_or_else(|_| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "Resolver future has been dropped",
                ))
            })
        }
            .boxed()
    }
}
