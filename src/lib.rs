mod config;

use config::Config;
use std::io::Result;
use std::net::IpAddr;
use std::net::UdpSocket;

pub struct GuestList {
    config: Config,
    nodes: Vec<Node>
}

/// Represents a Node in the cluster.
pub struct Node {
    addr: IpAddr,
    state: State,
}

/// A Node's possible states.
enum State {
    Alive,
    Suspected,
    Failed,
}

impl GuestList {

    pub fn start(self) -> Result<()> {
        let addr = format!("{}:{}", self.config.address, self.config.port);
        let socket = UdpSocket::bind(&addr)?;
        return Ok(());
    }
}
