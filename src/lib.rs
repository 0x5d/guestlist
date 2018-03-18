pub mod node;

/// Contains the node's configuration
pub struct Config {
    // The address to bind on
    address: String,
    // The port to listen on
    port: String,
    // The time to wait between failure detection "rounds"
    detection_period_ms: f64,
    //
    detection_ping_timeout: f64,
    // The number of random cluster members to contact each
    detection_group_size: u16,
}
