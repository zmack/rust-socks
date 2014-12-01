use std::os;
use std::{io, error};
use std::error::FromError;
use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener, IoError, IoResult};
use std::io::util::copy;
use std::io::net::addrinfo::get_host_addresses;
use std::io::net::ip::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::duration::Duration;

use server::SocksServer;
use configuration::Configuration;
use logger::Logger;
use client_tracker::{ClientTracker, ClientTrackers};

mod logger;
mod configuration;
mod client_tracker;
mod server;

fn main() {
    let args = os::args();
    let bind_address:&str;

    match args.len() {
        2 => {
            bind_address = args[1].as_slice();
        },
        _ => {
            bind_address = "127.0.0.1";
        },
    };
    let configuration = Configuration::new(Path::new("proxy.conf"));
    println!("whitelist -> {}", configuration.whitelisted_ips);
    println!("{} <-", os::args());
    let mut listener = TcpListener::bind(bind_address).unwrap();
    let socket_name = match listener.socket_name() {
        Ok(s) => s,
        _ => {
            println!("Error getting socket name");
            return
        }
    };
    println!("Listening on {}", socket_name);
    let logger = Logger::new();
    let trackers = ClientTrackers::new();

    let mut acceptor = listener.listen();

    for stream in acceptor.incoming() {
        let cloned_logger = logger.clone();
        let cloned_trackers = trackers.clone();
        match stream {
            Err(e) => {
                println!("There was an error omg {}", e)
            }
            Ok(stream) => {
                spawn(proc() {
                    SocksServer::new(stream, cloned_trackers, cloned_logger).handle_client();
                })
            }
        }
    }

    drop(acceptor);
}
