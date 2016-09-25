#![feature(lookup_host)]

#[macro_use]
extern crate log;

use std::os;
use std::{io, error};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::thread::spawn;

use server::SocksServer;
use configuration::Configuration;
use logger::Logger;
use client_tracker::{ClientTracker, ClientTrackers};

mod logger;
mod configuration;
mod client_tracker;
mod server;

fn main() {
    let args = std::env::args();
    let bind_address:&str;

    /*
    match args.len() {
        2 => {
            bind_address = &args.nth(1).unwrap();
        },
        _ => {
            bind_address = "127.0.0.1";
        },
    };
    */
    let configuration = Configuration::new();
    println!("whitelist -> {:?}", configuration.whitelisted_ips);
    let mut listener = TcpListener::bind("127.0.0.1:1090").unwrap();
    let socket_name = match listener.local_addr() {
        Ok(s) => s,
        _ => {
            println!("Error getting socket name");
            return
        }
    };
    println!("Listening on {:?}", socket_name);
    let logger = Logger::new();
    let trackers = ClientTrackers::new();

    loop {
        let cloned_logger = logger.clone();
        let cloned_trackers = trackers.clone();
        match listener.accept() {
            Err(e) => {
                println!("There was an error omg {}", e)
            }
            Ok((stream, remote)) => {
                spawn(move || {
                    SocksServer::new(stream, cloned_trackers, cloned_logger);
                });
            }
        }
    }
}
