// Declare sub modules
pub mod compute;
pub mod computed_metric;

// internal stuff
mod tests;  // needed to be part of the compilation unit in test mode


// Export our public api
pub use self::compute::compute_average;
pub use self::compute::compute_metric;
pub use self::compute::compute_p90;
pub use self::compute::compute_p99;
pub use self::compute::compute_p999;
pub use self::compute::sort_f64;
pub use self::computed_metric::ComputedMetric;
