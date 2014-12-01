use std::io::net::addrinfo::get_host_addresses;
use std::io::net::ip::{IpAddr, Ipv4Addr, SocketAddr};
use std::comm::{Sender, Receiver};
use std::io::File;
use std::sync::{Arc, Mutex};
use std::fmt;


#[deriving(Clone)]
struct ExecutionPool<'a, T> {
    sender: Sender<proc():Send>
}

impl<'a, T:Send<T>> ExecutionPool<'a, T> {
    pub fn new<'a, T>(size: uint) -> ExecutionPool<'a, T> {
        let (tx, rx) = channel::<proc():Send>();
        let receiver = Arc::new(Mutex::new(rx));

        for i in range(0u, size) {
            let cloned_rx = receiver.clone();
            let thread_id = i.clone();

            spawn(proc() {
                loop {
                    match cloned_rx.lock().recv_opt() {
                        Ok(x) => {
                            println!("Got a message on {}", thread_id);
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

    pub fn exec<T:Send<T>>(&self, fun:proc()) -> Result<T,String> {
        let (tx, rx) = channel();

        let my_proc:proc():Send = proc() {
            let res = fun();
            tx.send(res);
        };

        self.sender.send(my_proc);

        match rx.recv_opt() {
            Ok(x) => { return Ok(x) },
            _ => Err("Dead".to_string())
        }
    }
}

fn main() {
    let pool = ExecutionPool::new(10);

    for i in range(1u, 1_000u) {
        pool.exec(proc() -> int {
            println!("Hello");
            1
        });
    }
}

