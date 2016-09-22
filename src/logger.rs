use std::io::prelude::*;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::path::Path;
use std::fs::File;

enum Msg {
    Log(String)
}

#[derive(Clone)]
pub struct Logger {
    sender: Sender<Msg>,
}

impl Logger {
    pub fn new() -> Logger {
        let (tx, rx) = channel();
        ::std::thread::spawn(move || {
            Logger::perform_logging(rx);
        });
        return Logger {
            sender: tx,
        }
    }

    pub fn log(&self, message: &String) {
        self.sender.send(Msg::Log(message.clone()));
    }

    fn perform_logging(rx: Receiver<Msg>) {
        let mut file = File::create(&Path::new("urls.txt")).unwrap();
        loop {
            match rx.recv() {
                Ok(Msg::Log(message)) => {
                    println!("Got {}", message);
                    file.write(&message.into_bytes());
                }
            }
        }
        file.flush();
    }
}
