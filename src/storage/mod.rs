// Declare sub modules
pub mod macros;  // must be listed first since macros are order dependent

pub mod cache;
pub mod errors;
pub mod key;
pub mod typedefs;
pub mod value;

// internal stuff
mod accounting_hash_map;
mod tests;  // needed to be part of the compilation unit in test mode


// Export our public api
pub use self::cache::Cache;
pub use self::errors::CacheError;
pub use self::key::Key;
pub use self::typedefs::CacheResult;
pub use self::value::Value;
