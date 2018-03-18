use super::Config;
use std::io::Result;
use std::net::UdpSocket;

/// Represents a Node in the cluster.
pub struct Node {
    config: Config
}

impl Node {

    pub fn start(self) -> Result<()> {
        let addr = format!("{}:{}", self.config.address, self.config.port);
        let socket = UdpSocket::bind(&addr)?;
        return Ok(());
    }
}