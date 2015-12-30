#[derive(Debug, Clone)]
pub struct TransportStats {
    // These numbers are snapshots given that metrics are recorded concurrently
    // by each transport and transmitted to the protocol at regular intervals.
    pub bytes_read: u64,
    pub bytes_written: u64,
}

impl TransportStats {
    pub fn new() -> TransportStats {
        TransportStats {
            bytes_read: 0,
            bytes_written: 0,
        }
    }
}
