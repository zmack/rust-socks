use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::{IoError};
use std::io::util::copy;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::time::duration::Duration;

fn handle_client(mut tcp_stream: TcpStream) -> Result<(), IoError> {
    let addr = SocketAddr{ ip: Ipv4Addr(127, 0, 0, 1), port: 8000 };

    println!("addr {}", addr);

    let mut outbound = try!(TcpStream::connect_timeout(addr, Duration::seconds(5)));

    let mut client_in = tcp_stream.clone();
    let mut other_out = outbound.clone();

    spawn(proc() {
        let res_2 = copy(&mut client_in, &mut other_out);
        println!("2 {}", res_2);
    });

    let mut other_in = outbound.clone();
    let mut client_out = tcp_stream.clone();

    spawn(proc() {
        let res_1 = copy(&mut other_in, &mut client_out);
        println!("1 {}", res_1);
    });

    return Ok(());
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
