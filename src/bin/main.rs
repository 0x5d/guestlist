extern crate guestlist;

use guestlist::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn main() {
    let config = Config {
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000),
        detection_period_ms: 0.0,
        detection_ping_timeout: 0.0,
        detection_group_size: 0,
    };
    let g = Guestlist::with_config(config);
    match g.start() {
        Err(_) => return,
        Ok(handle) => handle.join(),
    };
}
