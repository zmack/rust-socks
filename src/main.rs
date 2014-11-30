use std::os;
use std::{io, error};
use std::error::FromError;
use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener, IoError, IoResult};
use std::io::util::copy;
use std::io::net::addrinfo::get_host_addresses;
use std::io::net::ip::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::duration::Duration;
use logger::Logger;
use client_tracker::{ClientTracker, ClientTrackers};
use configuration::Configuration;

mod logger;
mod configuration;
mod client_tracker;

enum RocksError {
    Io(IoError),
    Generic(String)
}

impl FromError<IoError> for RocksError {
    fn from_error(err: IoError) -> RocksError {
        RocksError::Io(err)
    }
}

impl FromError<Vec<u8>> for RocksError {
    fn from_error(err: Vec<u8>) -> RocksError {
        RocksError::Generic("Invalid utf encountered".to_string())
    }
}

impl FromError<String> for RocksError {
    fn from_error(string: String) -> RocksError {
        RocksError::Generic(string)
    }
}

fn authenticate(tcp_stream: &mut TcpStream, trackers: &ClientTrackers) -> Result<(), RocksError> {
    let version = try!(tcp_stream.read_u8());
    if version != 1 {
        return Err(FromError::from_error("Wrong version".to_string()))
    }
    let username_len = try!(tcp_stream.read_u8());
    let username_bytes = try!(tcp_stream.read_exact(username_len as uint));
    let username = try!(String::from_utf8(username_bytes));
    let password_len = try!(tcp_stream.read_u8());
    let password_bytes = try!(tcp_stream.read_exact(password_len as uint));
    let password = try!(String::from_utf8(password_bytes));

    trackers.track(&username);

    // Success
    tcp_stream.write(&[1, 0]);

    println!("Authentication credentials {} {}", username, password);

    Ok(())
}

fn handle_client(mut tcp_stream: TcpStream, logger: Logger, trackers: ClientTrackers) -> Result<(), RocksError> {
    loop {
        let version = try!(tcp_stream.read_u8());
        if version == 5 {
            let num_methods = try!(tcp_stream.read_u8());
            println!("Process command num {}", num_methods);
            let methods = try!(tcp_stream.read_exact(num_methods as uint));
            println!("Process command poo {} {}", num_methods, methods);
            if methods.contains(&2) {
                // Authenticated
                tcp_stream.write(&[5, 2]);
                try!(authenticate(&mut tcp_stream, &trackers));
            } else {
                // Unauthenticated
                tcp_stream.write(&[5, 0]);
            }
        } else {
            drop(tcp_stream);
            break
        }

        let v1 = try!(tcp_stream.read_u8());
        let c = try!(tcp_stream.read_u8());
        let res = try!(tcp_stream.read_u8());
        let addr_type = try!(tcp_stream.read_u8());

        let addr = try!(get_remote_addr(&mut tcp_stream, addr_type, &logger));
        
        println!("Process command {}", c);
        println!("res {}", res);
        println!("addr_type {}", addr_type);

        let mut outbound = try!(TcpStream::connect_timeout(addr, Duration::seconds(5)));
        println!("Connected {}", addr);

        try!(tcp_stream.write(&[5, 0, 0, 1, 127, 0, 0, 1, 0, 0]));

        let mut client_reader = tcp_stream.clone();
        let mut socket_writer = outbound.clone();

        spawn(proc() {
            let res_2 = copy(&mut client_reader, &mut socket_writer);
            client_reader.close_read();
            socket_writer.close_write();
        });

        let mut socket_reader = outbound.clone();
        let mut client_writer = tcp_stream.clone();

        let res_1 = copy(&mut socket_reader, &mut client_writer);
        socket_reader.close_read();
        client_writer.close_write();
    }

    return Ok(())
}

fn resolve_addr_with_cache(hostname: &str) -> Result<Vec<IpAddr>, String> {
    match get_host_addresses(hostname) {
        Ok(a) => { return Ok(a) },
        _ => { return Err("Done with this".to_string()) }
    };
}

fn get_remote_addr(tcp_stream: &mut TcpStream, addr_type: u8, logger: &Logger) -> Result<SocketAddr, RocksError> {
    match addr_type {
        1 => {
            let ip = try!(tcp_stream.read_exact(4));
            let port = try!(tcp_stream.read_be_uint_n(2)).to_u16().unwrap();

            return Ok(SocketAddr{ ip: Ipv4Addr(ip[0], ip[1], ip[2], ip[3]), port: port });
        },
        3 => {
            let num_str = try!(tcp_stream.read_u8()).to_uint().unwrap();
            println!("Reading {}", num_str);
            let hostname_vec = try!(tcp_stream.read_exact(num_str));
            let port = try!(tcp_stream.read_be_uint_n(2)).to_u16().unwrap();

            let hostname = match String::from_utf8(hostname_vec) { Ok(s) => s, _ => "".to_string() };
            logger.log(&hostname);
            let addresses = try!(resolve_addr_with_cache(hostname.as_slice()));

            if addresses.is_empty() {
                return Err(FromError::from_error("Empty Address".to_string()))
            } else {
                println!("Resolution succeeded for {} - {}", hostname, addresses);
                return Ok(SocketAddr{ ip: addresses[0], port: port });
            }
        },
        _ => return Err(FromError::from_error("Invalid Address Type".to_string()))
    }
}

fn process_command(command: u64, tcp_stream: &mut TcpStream) {
    println!("Command {} {}", command, command == 1u64);

    if command == 1u64 {
        let port = (*tcp_stream).read_le_uint_n(4);
        let ip = (*tcp_stream).read_le_uint_n(4);
    } else if command == 2u64 {
        println!("Some other command {}", command)
    }
}

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
                    handle_client(stream, cloned_logger, cloned_trackers);
                })
            }
        }
    }

    drop(acceptor);
}
