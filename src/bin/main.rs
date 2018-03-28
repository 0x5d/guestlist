extern crate guestlist;

use guestlist::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

fn main() {
    let config = Config {
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000),
        detection_period: Duration::from_millis(3000),
        detection_ping_timeout: Duration::from_millis(1000),
        detection_group_size: 2,
    };
    let g = Arc::new(Guestlist::with_config(config));
    let handles = Guestlist::start(g).unwrap_or_else(|_| panic!("Error"));

    for handle in handles {
        handle.join().unwrap();
    }
}
