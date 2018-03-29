use std::net::SocketAddr;
use std::time::Duration;

/// Configuration for a Guestlist instance.
pub struct Config {
    /// The address to bind on
    pub address: SocketAddr,
    /// Maximum time to wait before timing out when sending a message to another node.
    pub timeout: Duration,
    /// The time to wait between failure detection "rounds".
    pub detection_period: Duration,
    // The number of random cluster members to contact when a ping to a specific node fails.
    pub detection_group_size: u16,
}
