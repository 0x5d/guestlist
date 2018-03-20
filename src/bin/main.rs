extern crate guestlist;

use guestlist::*;
use std::net::{IpAddr, Ipv4Addr};

fn main() {
    let config = Config {
        address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port: "3000".to_owned(),
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
