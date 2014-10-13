use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};


fn handle_client(mut tcp_stream: TcpStream) {
    loop {
        let version = tcp_stream.read_le_uint_n(1);
        match version {
            Err(_) => break,
            Ok(v) => if v == 5 {
                let num_methods = tcp_stream.read_le_uint_n(1);
                match num_methods {
                    Err(e) => println!("Died due to {}", e),
                    Ok(num) => {
                        println!("Process command {}", num);
                        let methods = tcp_stream.read_le_uint_n(num as uint);
                        println!("Process command {} {}", num_methods, methods);
                        tcp_stream.write_le_uint(5);
                        tcp_stream.write_le_uint(0);
                    }
                }
            } else {
                drop(tcp_stream);
                break
            }
        }

        let command = tcp_stream.read_le_uint_n(1);

        match command {
            Err(e) => println!("Died due to {}", e),
            Ok(c) => {
                println!("Process command {}", c);
                let port = tcp_stream.read_le_uint_n(4);
                println!("Port {}", port);
                let ip = tcp_stream.read_le_uint_n(4);
                println!("Ip {}", ip);
                // process_command(c, &mut tcp_stream)
            }
        }
    }
}

fn process_command(command: u64, tcp_stream: &mut TcpStream) {
    println!("Command {} {}", command, command == 1u64);

    if command == 1u64 {
        let port = (*tcp_stream).read_le_uint_n(4);
        println!("Port {}", port);
        let ip = (*tcp_stream).read_le_uint_n(4);
        println!("Ip {}", ip);
    } else {
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
                handle_client(stream)
            })
        }
    }

    drop(acceptor);
}
