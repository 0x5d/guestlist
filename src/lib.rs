mod config;
mod messages;

extern crate rand;
extern crate rmp_serde;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub use config::Config;
use messages::Message;
use messages::Message::{Ack, Join, Ping};
use rand::{thread_rng, Rng};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::io::Result;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, RwLock};
use std::thread::{Builder, sleep, JoinHandle};

/// A Guestlist
pub struct Guestlist {
    /// This Guestlist instance's config.
    config: Config,
    /// A map where the key is the address <ip>:<port> and the value is a Node.
    nodes: RwLock<HashMap<String, Node>>,
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
            nodes: RwLock::new(HashMap::new()),
        }
    }

    /// Starts the UDP server so other nodes can ping the one running it or join the cluster.
    pub fn start(guestlist: Arc<Self>) -> Result<Vec<JoinHandle<()>>> {
        // FIXME: set a read timeout for this socket.
        let socket = UdpSocket::bind(&guestlist.config.address)?;

        let self1 = guestlist.clone();
        let self2 = self1.clone();

        // TODO: Figure out what to do with in-thread errors.
        let ping_handle = Builder::new()
            .name("ping_scheduler".to_owned())
            .spawn(move || self1.schedule_pings())?;

        let server_handle = Builder::new()
            .name("server".to_owned())
            .spawn(move || self2.run_server(socket))?;

        Ok(vec![ping_handle, server_handle])
    }

    fn schedule_pings(&self) {
        loop {
            // We create a block to drop the lock on the nodes map before putting the thread to sleep.
            {
                let nodes = self.nodes.read().unwrap();
                let nodes_length = nodes.len();
                if nodes_length > 0 {
                    let mut rng = thread_rng();
                    let i = if nodes_length == 1 {
                        0
                    } else {
                        rng.gen_range(0, nodes_length - 1)
                    };
                    // FIXME: It would be more time-efficient to have a Vec<Node> instead for O(1) access.
                    let node = nodes.values().nth(i).unwrap();
                    let this_addr = &self.config.address;
                    let this_ip = this_addr.ip();
                    let ping_msg = Ping { from: *this_addr };
                    let mut buf = Vec::new();
                    ping_msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                    // Bind on port 0 to get a random unused port.
                    let addr = format!("{}:0", this_ip);
                    let socket = UdpSocket::bind(&addr).unwrap();
                    socket.set_write_timeout(Some(self.config.detection_ping_timeout));
                    socket.set_read_timeout(Some(self.config.detection_ping_timeout));
                    socket.send_to(&buf, &node.address);
                    println!("pinging {}", node);
                }
            }
            sleep(self.config.detection_period);
        }
    }

    fn run_server(&self, socket: UdpSocket) {
        let mut buf = [0; 1000];

        loop {
            let (number_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("Didn't receive data");
            let mut deserializer = Deserializer::new(&buf[0..number_of_bytes]);
            let msg: Message = Deserialize::deserialize(&mut deserializer).unwrap();

            let mut reply_buf = Vec::new();
            let reply_msg = Ack { from: self.config.address };
            reply_msg.serialize(&mut Serializer::new(&mut reply_buf)).unwrap();

            let reply = match msg {
                Ping { from } => reply_msg,   
                Join { from } => reply_msg,
                _ => continue,
            };
            socket.send_to(&reply_buf, src_addr);
        }
    }
}
