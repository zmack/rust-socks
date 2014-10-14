use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::{IoError};
use std::io::util::copy;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::time::duration::Duration;


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
        
        println!("Process command {}", c);
        println!("res {}", res);
        println!("addr_type {}", addr_type);

        let ip = try!(tcp_stream.read_exact(4));
        let port = try!(tcp_stream.read_be_uint_n(2)).to_u16().unwrap();

        let addr = SocketAddr{ ip: Ipv4Addr(ip[0], ip[1], ip[2], ip[3]), port: port };

        println!("addr {}", addr);
        let mut outbound = try!(TcpStream::connect_timeout(addr, Duration::seconds(5)));
        try!(tcp_stream.write(&[5, 0, 0, 1, 127, 0, 0, 1, 0, 0]));

        // println!(">> {}", try!(tcp_stream.read_exact(4)));

        let mut client_reader = tcp_stream.clone();
        let mut socket_writer = outbound.clone();
        spawn(proc() {
            let res_2 = copy(&mut client_reader, &mut socket_writer);
        });

        let mut socket_reader = outbound.clone();
        let mut client_writer = tcp_stream.clone();

        let res_1 = copy(&mut socket_reader, &mut client_writer);


        // p[0], ip[1], ip[2], ip[3]rocess_command(c, &mut tcp_stream)
    }
    return Ok(())
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
                println!("Spawned a thing")
                handle_client(stream);
            })
        }
    }

    drop(acceptor);
}
