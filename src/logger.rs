use std::comm::{Sender, Receiver};
use std::io::File;

enum Msg {
    Log(String)
}

#[deriving(Clone)]
pub struct Logger {
    sender: Sender<Msg>,
}

impl Logger {
    pub fn new() -> Logger {
        let (tx, rx) = channel();
        spawn(proc() {
            Logger::perform_logging(rx);
        });
        return Logger {
            sender: tx,
        }
    }

    pub fn log(&self, message: &String) {
        self.sender.send(Log(message.clone()));
    }

    fn perform_logging(rx: Receiver<Msg>) {
        let mut file = File::create(&Path::new("urls.txt"));
        loop {
            match rx.recv() {
                Log(message) => {
                    println!("Got {}", message);
                    file.write_line(message.as_slice());
                }
            }
        }
        file.flush();
    }
}


#[test]
fn can_initialize() {
    let logger = Logger::new();
}
