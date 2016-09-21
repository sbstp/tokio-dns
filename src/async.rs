use std::io::{self, Read};
use std::fs::File;
use std::net;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use bytes::Buf;
use rand::{thread_rng, Rng};
use tokio_core::io::IoFuture;
use tokio_core::net::{ UdpSocket, VecBufferPool };
use tokio_core::net::stream::Udp;
use tokio_core::reactor::Handle;
use resolv_conf;
use dns_parser::{Packet, QueryType, QueryClass, RRData, Builder};
use futures::{oneshot, Complete, Future, finished};
use futures::stream::Stream;

use {Resolver};


/// An asynchronous resolver that uses tokio-core loop to resolve DNS packets
#[derive(Clone)]
pub struct AsyncResolver(Arc<Mutex<ResolverImpl>>);

struct ResolverImpl {
    config: resolv_conf::Config,
    socket: UdpSocket,
    handle: Handle,
    running: HashMap<u16, (String, Complete<Vec<net::IpAddr>>)>,
}

impl AsyncResolver {
    /// Initializes resolver from system config
    ///
    /// This usually means using `/etc/hosts` and `/etc/resolv.conf` on unix
    /// systems
    pub fn system_config(handle: Handle) -> Result<AsyncResolver, io::Error> {
        let mut buf = Vec::with_capacity(1024);
        let mut f = try!(File::open("/etc/resolv.conf"));
        try!(f.read_to_end(&mut buf));
        let config = try!(resolv_conf::Config::parse(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
        // Bind a random port
        let addr = net::SocketAddr::V4(net::SocketAddrV4::new(
                net::Ipv4Addr::new(0, 0, 0, 0), 0));
        let udp_socket = try!(UdpSocket::bind(&addr, &handle));
        let req_socket = try!(udp_socket.try_clone(&handle));
        let resolver = Arc::new(Mutex::new(ResolverImpl {
            config: config,
            socket: req_socket,
            handle: handle.clone(),
            running: HashMap::new(),
        }));
        let resolver2 = resolver.clone();
        handle.spawn(Udp::new(
            udp_socket,
            VecBufferPool::new(512),
        ).for_each(move |(buf, _socket_addr)| {
            accept_packet(buf.bytes(), &resolver2);
            Ok(())
        }).map_err(|_| ()));
        Ok(AsyncResolver(resolver))
    }
}

impl Resolver for AsyncResolver {
    fn resolve(&self, host: &str) -> IoFuture<Vec<net::IpAddr>> {
        let (complete, fut) = oneshot();
        self.send_request(host, complete);
        return Box::new(fut
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut,
                                        "future is cancelled")));
    }
}

impl AsyncResolver {
    fn send_request(&self, query: &str, future: Complete<Vec<net::IpAddr>>) {
        let copy = self.clone();
        let mut lock = self.0.lock().expect("async resolver locked");
        let mut id = thread_rng().gen();
        while lock.running.contains_key(&id) {
            id = thread_rng().gen();
        }

        let mut builder = Builder::new_query(id, true);
        builder.add_question(query, QueryType::A, QueryClass::IN);

        // TODO(tailhook) this may fail if query is too long
        let pack = builder.build().expect("packet should be built");
        // TODO(tailhook) check if there is at least one nameserver
        let server = lock.config.nameservers[0];

        lock.running.insert(id, (query.to_string(), future));
        lock.handle.spawn(finished(()).and_then(move |()| {
            let lock = copy.0.lock().expect("async resolver locked");
            lock.socket.send_to(&pack, &net::SocketAddr::new(server, 53))
        }).map(|_| ()).map_err(|_| ()));
    }
}

fn accept_packet(data: &[u8], resolver: &Arc<Mutex<ResolverImpl>>) {
    let pack = match Packet::parse(data) {
        Ok(pack) => pack,
        Err(_) => {
            // Just a bad packet. Should we log it?
            return;
        }
    };
    let id = pack.header.id;
    let mut lock = resolver.lock().expect("resolver locked");
    let (query, fut) = match lock.running.remove(&id) {
        Some(request) => request,
        None => {
            // Unsolicited reply. Should we log it?
            return;
        }
    };
    // TODO(tailhook) check server
    if pack.questions.len() != 1 ||
            pack.questions[0].qtype != QueryType::A ||
            pack.questions[0].qclass != QueryClass::IN ||
            pack.questions[0].qname.to_string() != query
    {
        // Probably someone tries to spoof us. Log it?
        lock.running.insert(id, (query, fut));
        return;
    }
    let mut ips = Vec::with_capacity(pack.answers.len());
    for ans in pack.answers {
        match ans.data {
            RRData::A(ip) => {
                ips.push(net::IpAddr::V4(ip));
            }
            _ => {
                // Bad value. Log it?
            }
        }
    }
    fut.complete(ips);
}
