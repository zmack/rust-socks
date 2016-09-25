use std::net::lookup_host;
use std::net::{SocketAddr};
use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;

#[derive(Clone)]
pub struct DnsCache {
    tx: Sender<ResolveRequest>
}

struct ResolveRequest {
    hostname: String,
    tx: Sender<ResolveResponse>
}

struct ResolveResponse {
    addr: Option<SocketAddr>
}

impl DnsCache {
    pub fn new() -> DnsCache {
        let (tx, rx) = channel::<ResolveRequest>();

        let mut cache = DnsCache {
            tx: tx
        };

        let mut store = HashMap::new();

        thread::spawn(move || {
            handle_message(store, rx);
        });

        cache
    }

    pub fn resolve(&self, hostname: &str) -> Option<SocketAddr> {
        let (tx, rx) = channel::<ResolveResponse>();
        let request = ResolveRequest { hostname: hostname.to_string(), tx: tx };
        self.tx.send(request);
        rx.recv().unwrap().addr
    }
}

fn handle_message(mut store: HashMap<String, Option<SocketAddr>>, rx: Receiver<ResolveRequest>) {
    loop {
        let message;
        match rx.recv() {
            Ok(m) => { message = m },
            _ => continue
        };

        let hostname = message.hostname;
        let value = store.entry(hostname.clone()).or_insert_with(move || {
            match lookup_host(&hostname) {
                Ok(mut a) => { a.nth(0) },
                _ => { None }
            }
        });
        message.tx.send(ResolveResponse { addr: *value });
    }

}
