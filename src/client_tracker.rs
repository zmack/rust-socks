use std::comm::{Sender, Receiver};
use std::io::File;
use std::collections::HashMap;

enum Msg {

}

enum ClientTrackersMsg {
    Get(String),
}

#[deriving(Clone)]
pub struct ClientTracker {
    client: String,
    total_traffic: u64,
    sender: Sender<Msg>
}

#[deriving(Clone)]
pub struct ClientTrackers {
    sender: Sender<ClientTrackersMsg>
}

impl ClientTrackers {
    pub fn new() -> ClientTrackers {
        let (tx, rx) = channel();

        spawn(proc() {
            ClientTrackers::serve(rx);
        });

        ClientTrackers {
            sender: tx
        }
    }

    fn serve(rx: Receiver<ClientTrackersMsg>) {
        let mut trackers = ClientTrackersInner::new();

        loop {
            match rx.recv() {
                ClientTrackersMsg::Get(key) => {
                    let tracker = trackers.get(key);
                    tracker.increment();
                }
            }
        }
    }

    pub fn track(&self, key: &String) {
        self.sender.send(ClientTrackersMsg::Get(key.clone()));
    }
}


struct ClientTrackersInner {
    num_trackers: u64,
    trackers: HashMap<String, ClientTracker>
}

impl ClientTrackersInner {
    pub fn new() -> ClientTrackersInner {
        ClientTrackersInner {
            num_trackers: 0,
            trackers: HashMap::new()
        }
    }

    pub fn get(&mut self, client: String) -> ClientTracker {
        let trackers = &self.trackers.clone();
        let client_key = client.clone();
        let found_tracker = trackers.find(&client.clone());
        match found_tracker {
            Some(c) => return (*c).clone(),
            _ => {
                let tracker = ClientTracker::new(client);
                &self.add_tracker(client_key, tracker.clone());
                tracker
            }
        }
    }

    pub fn add_tracker(&mut self, client: String, tracker: ClientTracker) {
        self.num_trackers += 1;
        println!("Trackers {}", self.num_trackers);
        self.trackers.insert(client, tracker);
    }
}

impl ClientTracker {
    pub fn new(client: String) -> ClientTracker {
        let (tx, rx) = channel();
        ClientTracker {
            client: client,
            total_traffic: 0,
            sender: tx
        }
    }

    pub fn increment(&self) {
        println!("Increment!");
    }
}

#[test]
fn can_initialize() {
    let tracker = ClientTracker::new("Hello".to_string());
}
