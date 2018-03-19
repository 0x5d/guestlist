mod config;

pub use config::Config;
use std::io::Result;
use std::net::IpAddr;
use std::net::UdpSocket;
use std::thread::{JoinHandle, spawn};

pub struct Guestlist {
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

impl Guestlist {

    pub fn with_config(config: Config) -> Guestlist {
        let mut nodes = Vec::new(); 
        Guestlist {
            config: config,
            nodes: nodes
        }
    }

    pub fn start(self) -> Result<JoinHandle<String>> {
        let addr = format!("{}:{}", self.config.address, self.config.port);
        let socket = UdpSocket::bind(&addr)?;

        let handle = spawn(move || {
            let mut buf = [0; 1000];

            loop {
                let (number_of_bytes, src_addr) = socket.recv_from(&mut buf)
                    .expect("Didn't receive data");
                let msg = String::from_utf8(buf[0 .. number_of_bytes].to_vec());

                match msg {
                    Ok(m) => match m.as_ref() {
                        "ping" => socket.send_to("alive".as_bytes(), src_addr),
                        "join" => socket.send_to("joined".as_bytes(), src_addr),
                        _ => continue,
                    },
                    Err(_) => continue,
                };
            }
        });
        return Ok((handle));
    }
}
