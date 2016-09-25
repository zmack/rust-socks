extern crate byteorder;

use std::os;
use std::{io, error};
use std::convert::From;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Error,copy};
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;

use logger::Logger;
use dns_cache::DnsCache;
use client_tracker::{ClientTracker, ClientTrackers};
use configuration::Configuration;

use server::byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

enum RocksError {
    Io(Error),
    Generic(String)
}

pub struct SocksServer {
    trackers: ClientTrackers,
    tcp_stream: TcpStream,
    logger: Logger,
    dns: DnsCache,
}

impl SocksServer {
    pub fn new(tcp_stream: TcpStream, trackers:ClientTrackers, logger: Logger, dns: DnsCache) {
        let mut server = SocksServer {
            tcp_stream: tcp_stream,
            trackers: trackers,
            dns: dns,
            logger: logger
        };
        server.handle_client();
    }

    fn handle_client(&mut self) -> Result<(), RocksError> {
        loop {
            let version;
            match self.tcp_stream.read_u8() {
                Ok(v) => { version = v },
                _ => break
            }
            if version == 5 {
                let num_methods = self.tcp_stream.read_u8().unwrap();
                let mut methods = Vec::with_capacity(num_methods as usize);
                unsafe { methods.set_len(num_methods as usize) };
                self.tcp_stream.read_exact(&mut methods).unwrap();
                debug!("num_methods is {:?}, methods is {:?}", num_methods, methods);

                if methods.contains(&2) {
                    // Authenticated
                    self.tcp_stream.write(&[5, 2]);
                    self.authenticate().unwrap()
                } else {
                    // Unauthenticated
                    self.tcp_stream.write(&[5, 0]);
                }
            } else {
                drop(&self.tcp_stream);
                break
            }

            let v1 = self.tcp_stream.read_u8().unwrap();
            let c = self.tcp_stream.read_u8().unwrap();
            let res = self.tcp_stream.read_u8().unwrap();
            let addr_type = self.tcp_stream.read_u8().unwrap();

            debug!("v1 is {:?}", v1);
            debug!("c is {:?}", c);
            debug!("res is {:?}", res);
            debug!("Address type is {:?}", addr_type);
            let addr = self.get_remote_addr(addr_type).unwrap();

            debug!("Address is {:?}", addr);

            let mut outbound = TcpStream::connect(addr).unwrap();
            outbound.set_read_timeout(Some(Duration::from_secs(5))).unwrap();

            self.tcp_stream.write(&[5, 0, 0, 1, 127, 0, 0, 1, 0, 0]).unwrap();
            debug!("Wrote things");

            let mut client_reader = self.tcp_stream.try_clone().unwrap();
            debug!("Clone reader");
            let mut socket_writer = outbound.try_clone().unwrap();
            debug!("Clone writer");

            // Copy doesn't return total bytes copied.
            // Either roll our own moving forward or just scrap tracking
            ::std::thread::spawn(move || {
                copy(&mut client_reader, &mut socket_writer);
                client_reader.shutdown(Shutdown::Read);
                socket_writer.shutdown(Shutdown::Write);
            });

            let mut socket_reader = outbound.try_clone().unwrap();
            let mut client_writer = self.tcp_stream.try_clone().unwrap();

            copy(&mut socket_reader, &mut client_writer);
            socket_reader.shutdown(Shutdown::Read);
            client_writer.shutdown(Shutdown::Write);
        }

        return Ok(())
    }

    fn authenticate(&mut self) -> Result<(), String> {
        let version = self.tcp_stream.read_u8().unwrap();
        if version != 1 {
            return Err(From::from("Wrong version".to_string()))
        }
        let username_len = self.tcp_stream.read_u8().unwrap();
        let mut username_vec = Vec::with_capacity(username_len as usize);
        self.tcp_stream.read_exact(&mut username_vec);
        let username = String::from_utf8(username_vec).unwrap();
        let password_len = self.tcp_stream.read_u8().unwrap();
        let mut password_vec = Vec::with_capacity(password_len as usize);
        self.tcp_stream.read_exact(&mut password_vec).unwrap();
        let password = String::from_utf8(password_vec).unwrap();

        self.trackers.track(&username);

        // Success
        self.tcp_stream.write(&[1, 0]);

        debug!("Authentication credentials {} {}", username, password);

        Ok(())
    }

    fn get_remote_addr(&mut self, addr_type: u8) -> Result<SocketAddr, String> {
        match addr_type {
            1 => {
                let mut ip_bytes = [0u8; 4];
                self.tcp_stream.read_exact(&mut ip_bytes);
                let ip = Ipv4Addr::from(ip_bytes);
                let port = self.tcp_stream.read_u16::<BigEndian>().unwrap();

                return Ok(SocketAddr::V4(SocketAddrV4::new(ip, port)));
            },
            3 => {
                let num_str = self.tcp_stream.read_u8().unwrap();
                let mut hostname_vec = Vec::with_capacity(num_str as usize);
                unsafe { hostname_vec.set_len(num_str as usize) };
                self.tcp_stream.read_exact(&mut hostname_vec).unwrap();
                let port = self.tcp_stream.read_u16::<BigEndian>().unwrap();

                let hostname = match String::from_utf8(hostname_vec) { Ok(s) => s, _ => "".to_string() };
                self.logger.log(hostname.clone());
                let address = self.resolve_addr_with_cache(&hostname);

                if address.is_none() {
                    return Err(From::from("Empty Address".to_string()))
                } else {
                    // println!("Resolution succeeded for {?} - {?}", hostname, addresses);
                    let mut address = address.unwrap();
                    address.set_port(port);
                    return Ok(address);
                }
            },
            _ => return Err(From::from("Invalid Address Type".to_string()))
        }
    }

    fn resolve_addr_with_cache(&self, hostname: &str) -> Option<SocketAddr> {
        self.dns.resolve(hostname)
    }
}


