// Declare sub modules
pub mod errors;
pub mod metrics;
pub mod transport;
pub mod typedefs;

// internal stuff
mod test_stream;
mod tests;  // needed to be part of the compilation unit in test mode


// Export our public api
pub use self::errors::TcpTransportError;
pub use self::metrics::TransportMetrics;
pub use self::transport::TcpTransport;
pub use self::typedefs::TcpTransportResult;
