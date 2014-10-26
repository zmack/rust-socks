use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener, IoError, IoResult};
use std::io::util::copy;
use std::io::net::addrinfo::get_host_addresses;
use std::io::net::ip::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::duration::Duration;
use logger;


fn handle_client(mut tcp_stream: TcpStream) -> Result<(), IoError> {
    loop {
        let version = tcp_stream.read_le_uint_n(1);
        match version {
            Err(_) => break,
            Ok(v) => if v == 5 {
                let num_methods = try!(tcp_stream.read_le_uint_n(1));
                println!("Process command num {}", num_methods);
                let methods = try!(tcp_stream.read_be_uint_n(num_methods as uint));
                println!("Process command poo {} {:X}", num_methods, methods);
                tcp_stream.write([5, 0]);
                //tcp_stream.write_le_uint(0);
            } else {
                drop(tcp_stream);
                break
            }
        }

        let v1 = try!(tcp_stream.read_le_uint_n(1));
        let c = try!(tcp_stream.read_le_uint_n(1));
        let res = try!(tcp_stream.read_le_uint_n(1));
        let addr_type = try!(tcp_stream.read_le_uint_n(1));

        let addr = try!(get_remote_addr(&mut tcp_stream, addr_type));
        
        println!("Process command {}", c);
        println!("res {}", res);
        println!("addr_type {}", addr_type);

        let mut outbound = try!(TcpStream::connect_timeout(addr, Duration::seconds(5)));
        println!("Connected {}", addr);

        try!(tcp_stream.write(&[5, 0, 0, 1, 127, 0, 0, 1, 0, 0]));

        let mut client_reader = tcp_stream.clone();
        let mut socket_writer = outbound.clone();

        println!("Started copy");
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

fn get_remote_addr(tcp_stream: &mut TcpStream, addr_type: u64) -> IoResult<SocketAddr> {
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
            let addresses = match resolve_addr_with_cache(hostname.as_slice()) {
                Ok(a) => a,
                _ => return Err(IoError::last_error())
            };

            if addresses.is_empty() {
                return Err(IoError::last_error())
            } else {
                println!("Resolution succeeded for {} - {}", hostname, addresses);
                return Ok(SocketAddr{ ip: addresses[0], port: port });
            }
        },
        _ => return Err(IoError::last_error())
    }
}

fn process_command(command: u64, tcp_stream: &mut TcpStream) {
    println!("Command {} {}", command, command == 1u64);

    if command == 1u64 {
        let port = (*tcp_stream).read_le_uint_n(4);
        println!("Port {}", port);
        let ip = (*tcp_stream).read_le_uint_n(4);
        println!("Ip {}", ip);
    } else if command == 2u64 {
        println!("Some other command {}", command)
    }

    println!("Got command {}", command)
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1", 1080);

    let mut acceptor = listener.listen();

    for stream in acceptor.incoming() {
        match stream {
            Err(e) => {
                println!("There was an error omg {}", e)
            }
            Ok(stream) => spawn(proc() {
                handle_client(stream);
            })
        }
    }

    drop(acceptor);
}
