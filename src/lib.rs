mod config;

pub use config::Config;
use std::collections::HashMap;
use std::fmt;
use std::io::Result;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread::{sleep, spawn, JoinHandle};

pub struct Guestlist {
    config: Mutex<Config>,
    // A map where the key is the address <ip>:<port> and the value is a Node.
    nodes: HashMap<String, Node>,
}

/// Represents a Node in the cluster.
#[derive(Debug)]
pub struct Node {
    address: SocketAddr,
    state: State,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.address, self.state)
    }
}

/// A Node's possible states.
enum State {
    Alive,
    Suspected,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            &State::Alive => "alive",
            &State::Suspected => "suspected",
        };
        write!(f, "{}", s)
    }
}
impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self as &fmt::Display).fmt(f)
    }
}

impl Guestlist {
    pub fn with_config(config: Config) -> Guestlist {
        Guestlist {
            config: Mutex::new(config),
            nodes: HashMap::new(),
        }
    }

    pub fn start(self) -> Result<JoinHandle<()>> {
        let addr = format!("{}", self.config.lock().unwrap().address);
        let socket = UdpSocket::bind(&addr)?;

        let this = Arc::new(self);
        let this_server = Arc::clone(&this);

        spawn(move || this.schedule_pings());

        let handle = spawn(move || this_server.run_server(socket));
        return Ok(handle);
    }

    fn schedule_pings(&self) {
        loop {
            println!("ping");
            let config = self.config.lock().unwrap();
            sleep(Duration::from_millis(config.detection_period_ms));
        }
    }

    fn run_server(&self, socket: UdpSocket) {
        let mut buf = [0; 1000];

        loop {
            let (number_of_bytes, src_addr) = 
                socket.recv_from(&mut buf).expect("Didn't receive data");
            let msg = String::from_utf8(buf[0..number_of_bytes].to_vec());

            match msg {
                Ok(m) => {
                    let trimmed = m.trim();
                    let nodes_str = format!("{:?}", &self.nodes.values());
                    let reply = match trimmed.as_ref() {
                        "ping" => "alive",
                        "join" => nodes_str.as_ref(),
                        _ => continue,
                    };
                    socket.send_to(reply.as_bytes(), src_addr);
                }
                Err(_) => continue,
            };
        }
    }
}
