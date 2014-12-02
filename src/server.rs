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

enum RocksError {
    Io(IoError),
    Generic(String)
}

pub struct SocksServer {
    trackers: ClientTrackers,
    tcp_stream: TcpStream,
    logger: Logger
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

impl SocksServer {
    pub fn new(tcp_stream: TcpStream, trackers:ClientTrackers, logger: Logger) {
        let mut server = SocksServer {
            tcp_stream: tcp_stream,
            trackers: trackers,
            logger: logger
        };
        server.handle_client();
    }

    fn handle_client(&mut self) -> Result<(), RocksError> {
        loop {
            let version = try!(self.tcp_stream.read_u8());
            if version == 5 {
                let num_methods = try!(self.tcp_stream.read_u8());
                let methods = try!(self.tcp_stream.read_exact(num_methods as uint));

                if methods.contains(&2) {
                    // Authenticated
                    self.tcp_stream.write(&[5, 2]);
                    try!(self.authenticate());
                } else {
                    // Unauthenticated
                    self.tcp_stream.write(&[5, 0]);
                }
            } else {
                drop(&self.tcp_stream);
                break
            }

            let v1 = try!(self.tcp_stream.read_u8());
            let c = try!(self.tcp_stream.read_u8());
            let res = try!(self.tcp_stream.read_u8());
            let addr_type = try!(self.tcp_stream.read_u8());

            let addr = try!(self.get_remote_addr(addr_type));

            let mut outbound = try!(TcpStream::connect_timeout(addr, Duration::seconds(5)));

            try!(self.tcp_stream.write(&[5, 0, 0, 1, 127, 0, 0, 1, 0, 0]));

            let mut client_reader = self.tcp_stream.clone();
            let mut socket_writer = outbound.clone();

            // Copy doesn't return total bytes copied.
            // Either roll our own moving forward or just scrap tracking
            spawn(proc() {
                copy(&mut client_reader, &mut socket_writer);
                client_reader.close_read();
                socket_writer.close_write();
            });

            let mut socket_reader = outbound.clone();
            let mut client_writer = self.tcp_stream.clone();

            copy(&mut socket_reader, &mut client_writer);
            socket_reader.close_read();
            client_writer.close_write();
        }

        return Ok(())
    }

    fn authenticate(&mut self) -> Result<(), RocksError> {
        let version = try!(self.tcp_stream.read_u8());
        if version != 1 {
            return Err(FromError::from_error("Wrong version".to_string()))
        }
        let username_len = try!(self.tcp_stream.read_u8());
        let username_bytes = try!(self.tcp_stream.read_exact(username_len as uint));
        let username = try!(String::from_utf8(username_bytes));
        let password_len = try!(self.tcp_stream.read_u8());
        let password_bytes = try!(self.tcp_stream.read_exact(password_len as uint));
        let password = try!(String::from_utf8(password_bytes));

        self.trackers.track(&username);

        // Success
        self.tcp_stream.write(&[1, 0]);

        println!("Authentication credentials {} {}", username, password);

        Ok(())
    }

    fn get_remote_addr(&mut self, addr_type: u8) -> Result<SocketAddr, RocksError> {
        match addr_type {
            1 => {
                let ip = try!(self.tcp_stream.read_exact(4));
                let port = try!(self.tcp_stream.read_be_uint_n(2)).to_u16().unwrap();

                return Ok(SocketAddr{ ip: Ipv4Addr(ip[0], ip[1], ip[2], ip[3]), port: port });
            },
            3 => {
                let num_str = try!(self.tcp_stream.read_u8()).to_uint().unwrap();
                let hostname_vec = try!(self.tcp_stream.read_exact(num_str));
                let port = try!(self.tcp_stream.read_be_uint_n(2)).to_u16().unwrap();

                let hostname = match String::from_utf8(hostname_vec) { Ok(s) => s, _ => "".to_string() };
                self.logger.log(&hostname);
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
}


fn resolve_addr_with_cache(hostname: &str) -> Result<Vec<IpAddr>, String> {
    match get_host_addresses(hostname) {
        Ok(a) => { return Ok(a) },
        _ => { return Err("Done with this".to_string()) }
    };
}
