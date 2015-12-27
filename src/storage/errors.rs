#[derive(Debug, PartialEq)]
pub enum CacheError {
    CapacityExceeded,
    EvictionFailed,
    KeyNotFound,
    KeyTooLong,
    ValueTooLong,
}
