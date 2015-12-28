#[derive(Debug, Clone)]
pub struct TransportMetrics {
    // These numbers are snapshots given that metrics are recorded concurrently
    // by each transport and transmitted to the protocol at regular intervals.
    pub bytes_read: u64,
    pub bytes_written: u64,
}

impl TransportMetrics {
    pub fn new() -> TransportMetrics {
        TransportMetrics {
            bytes_read: 0,
            bytes_written: 0,
        }
    }
}
