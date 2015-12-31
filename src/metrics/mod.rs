// Declare sub modules
pub mod live_timers;
pub mod metric;
pub mod metrics;
pub mod recorder;
pub mod statistics;
pub mod time_series;
pub mod timer;
pub mod timing;
pub mod typedefs;

// internal stuff
mod tests;  // needed to be part of the compilation unit in test mode


// Export our public api
pub use self::live_timers::LiveTimers;
pub use self::metric::Metric;
pub use self::metrics::Metrics;
pub use self::recorder::MetricsRecorder;
pub use self::time_series::TimeSeries;
pub use self::timer::Timer;
pub use self::timing::Timing;
pub use self::typedefs::Duration;
pub use self::typedefs::Second;
pub use self::typedefs::StartTime;
