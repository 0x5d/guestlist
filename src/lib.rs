mod domain;

extern crate rand;
extern crate rmp_serde;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub use domain::config::Config;
use domain::{Node, State};
use domain::error::GuestlistError;
use domain::message::Message;
use domain::message::Message::{Ack, Join, Ping};

use rand::{thread_rng, Rng};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, RwLock};
use std::thread::{Builder, sleep, JoinHandle};

type GuestlistResult<T> = Result<T, GuestlistError>;

/// A Guestlist
pub struct Guestlist {
    /// This Guestlist instance's config.
    config: Config,
    /// A map where the key is the address <ip>:<port> and the value is a Node.
    nodes: RwLock<HashMap<String, Node>>,
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
    pub fn start(guestlist: Arc<Self>) -> GuestlistResult<Vec<JoinHandle<()>>> {
        let self1 = guestlist.clone();
        let self2 = self1.clone();

        // TODO: Figure out what to do with in-thread errors.
        let ping_handle = Builder::new()
            .name("ping_scheduler".to_owned())
            .spawn(move || self1.schedule_pings())?;

        let server_handle = Builder::new()
            .name("server".to_owned())
            .spawn(move || self2.run_server())?;

        Ok(vec![ping_handle, server_handle])
    }

    pub fn join(&self, address: SocketAddr) -> GuestlistResult<()> {
        let msg = Join { from: self.config.address };
        self.send_message(msg, address)
    }

    fn schedule_pings(&self) {
        loop {
            // We create a block to drop the read lock on the nodes map before putting the thread to sleep.
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
                    self.send_ping(node.address).unwrap();
                    println!("pinging {}", node);
                }
            }
            sleep(self.config.detection_period);
        }
    }

    fn run_server(&self) {
        let socket = UdpSocket::bind(self.config.address).unwrap();
        socket.set_write_timeout(Some(self.config.timeout)).unwrap();
        let mut buf = [0; 1000];

        loop {
            let (number_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("Didn't receive data");
            let mut deserializer = Deserializer::new(&buf[0..number_of_bytes]);
            let msg: Message = Deserialize::deserialize(&mut deserializer).unwrap();

            match msg {
                Ping { from } => self.send_ack(src_addr),
                Join { from } => self.add_node(from),
                _ => continue,
            };
        }
    }

    fn add_node(&self, address: SocketAddr) -> GuestlistResult<()> {
        let node = Node {
            address: address,
            state: State::Alive
        };
        let mut ns = self.nodes.write()?;
        let address_str = address.to_string();
        let n = ns.entry(address_str).or_insert(node);
        n.state = State::Alive;
        Ok(())
    }

    fn send_ping(&self, address: SocketAddr) -> GuestlistResult<()> {
        let msg = Ping { from: self.config.address };
        self.send_message(msg, address)
    }

    fn send_ack(&self, address: SocketAddr) -> GuestlistResult<()> {
        let msg = Ack { from: self.config.address };
        self.send_message(msg, address)
    }

    fn send_message(&self, msg: Message, address: SocketAddr) -> GuestlistResult<()> {
        let mut buf = Vec::new();
        msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
        // Bind on port 0 to get a random unused port.
        let addr = format!("{}:0", self.config.address.ip());
        let socket = UdpSocket::bind(&addr)?;
        socket.set_write_timeout(Some(self.config.timeout))?;
        socket.send_to(&buf, address).map(|_| ())?;
        Ok(())
    }
}
