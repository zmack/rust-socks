use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::str;


fn handle_client(mut tcp_stream: TcpStream) {
    let mut buffer:[u8, ..1024] = [0, ..1024];

    loop {
        tcp_stream.read(buffer);
        let s = match str::from_utf8(buffer) {
            None => {
                println!("Dude went away")
                break;
            },
            Some(e) => e,
        };
        println!("Got -> {}", s);
    }
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
                handle_client(stream)
            })
        }
    }

    drop(acceptor);
}
