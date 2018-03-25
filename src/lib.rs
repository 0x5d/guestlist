mod config;

extern crate rand;

pub use config::Config;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::fmt;
use std::io::Result;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread::{sleep, spawn, JoinHandle};

/// A Guestlist 
pub struct Guestlist {
    /// This Guestlist instance's config.
    config: Config,
    /// A map where the key is the address <ip>:<port> and the value is a Node.
    nodes: Mutex<HashMap<String, Node>>,
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

    /// Creates a Guestlist instance with the provided configuration.
    pub fn with_config(config: Config) -> Guestlist {
        Guestlist {
            config: config,
            nodes: Mutex::new(HashMap::new()),
        }
    }

    /// Starts the UDP server so other nodes can ping the one running it or join the cluster.
    pub fn start(self) -> Result<JoinHandle<()>> {
        let socket = UdpSocket::bind(&self.config.address)?;

        let this = Arc::new(self);
        let this_server = Arc::clone(&this);

        // TODO: Figure out what to do with in-thread errors.
        spawn(move || this.schedule_pings());

        let handle = spawn(move || this_server.run_server(socket));
        return Ok(handle);
    }

    fn schedule_pings(&self) {
        loop {
            let nodes = self.nodes.lock().unwrap();
            let nodes_length = nodes.len();
            if nodes_length > 0 {
                let mut rng = thread_rng();
                let i = rng.gen_range(0, nodes_length - 1);
                // FIXME: It would be more time-efficient to have a Vec<Node> instead for O(1) access.
                let node = nodes.values().nth(i).unwrap();
                let this_ip = &self.config.address.ip();
                // Bind on port 0 to get a random unused port.
                let addr = format!("{}:0", this_ip);
                let socket = UdpSocket::bind(&addr).unwrap();
                socket.send_to("ping".as_bytes(), &node.address);
                println!("pinging {}", node);
            }
            sleep(Duration::from_millis(self.config.detection_period_ms));
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
                    let nodes_str = format!("{:?}", &self.nodes.lock().unwrap().values());
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
