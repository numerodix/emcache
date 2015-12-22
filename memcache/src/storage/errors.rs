#[derive(Debug, PartialEq)]
pub enum CacheError {
    CapacityExceeded,
    KeyNotFound,
    KeyTooLong,
    ValueTooLong,
}

