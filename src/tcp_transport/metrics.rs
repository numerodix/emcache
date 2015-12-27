#[derive(Debug, Clone)]
pub struct TransportMetrics {
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
