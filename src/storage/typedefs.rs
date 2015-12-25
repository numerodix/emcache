use super::errors::CacheError;


pub type CacheResult<T> = Result<T, CacheError>;
