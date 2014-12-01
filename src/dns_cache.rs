use std::io::net::addrinfo::get_host_addresses;
use std::io::net::ip::{IpAddr, Ipv4Addr, SocketAddr};
use std::comm::{Sender, Receiver};
use std::io::File;
use std::sync::{Arc, Mutex};
use std::fmt;


#[deriving(Clone)]
struct ExecutionPool {
    sender: Sender<proc():Send>
}

impl ExecutionPool {
    pub fn new(size: uint) -> ExecutionPool {
        let (tx, rx) = channel::<proc():Send>();
        let receiver = Arc::new(Mutex::new(rx));

        for i in range(0u, size) {
            let cloned_rx = receiver.clone();
            let thread_id = i.clone();

            spawn(proc() {
                loop {
                    match cloned_rx.lock().recv_opt() {
                        Ok(x) => {
                            x();
                        },
                        _ => {},
                    }
                }
            });
        }

        ExecutionPool {
            sender: tx
        }
    }

    pub fn exec(&self, fun:proc():Send) {
        self.sender.send(fun);
    }
}

fn main() {
    let pool = ExecutionPool::new(10);

    for i in range(1u, 1_000u) {
        let (tx, rx) = channel::<uint>();
        pool.exec(proc() {
            println!("Hello");
            tx.send(10);
        });

        println!("Done with {} -> {}", i, rx.recv());
    }
}

