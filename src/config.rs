use std::net::SocketAddr;
use std::time::Duration;

/// Configuration for a Guestlist instance.
pub struct Config {
    /// The address to bind on
    pub address: SocketAddr,
    /// The time to wait between failure detection "rounds".
    pub detection_period: Duration,
    /// Maximum time to wait before timing out when pinging another node.
    pub detection_ping_timeout: Duration,
    // The number of random cluster members to contact when a ping to a specific node fails.
    pub detection_group_size: u16,
}
