// Declare sub modules
pub mod aggregate;
pub mod aggregated_metric;

// internal stuff
mod tests;  // needed to be part of the compilation unit in test mode


// Export our public api
pub use self::aggregate::aggregate_metric;
pub use self::aggregate::compute_average;
pub use self::aggregate::compute_p0;
pub use self::aggregate::compute_p90;
pub use self::aggregate::compute_p99;
pub use self::aggregate::compute_p999;
pub use self::aggregate::sort_f64;
pub use self::aggregated_metric::AggregatedMetric;
