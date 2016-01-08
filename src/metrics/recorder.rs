use orchestrator::MetricsSender;

use super::LiveTimers;
use super::Metric;
use super::Metrics;


pub struct MetricsRecorder {
    // If not enabled the recorder is just a shim - records nothing, transmits
    // nothing (no performance overhead)
    // TODO: Consider replacing this flag with a Collector trait - can implement
    // a real collector and a NullCollector
    enabled: bool,

    live_timers: LiveTimers,
    metrics: Metrics,

    met_tx: MetricsSender,
}

impl MetricsRecorder {
    pub fn new(met_tx: MetricsSender, enabled: bool) -> MetricsRecorder {
        MetricsRecorder {
            enabled: enabled,
            live_timers: LiveTimers::new(),
            metrics: Metrics::new(),
            met_tx: met_tx,
        }
    }

    pub fn time_it<T>(&mut self, name: &str, closure: &mut FnMut() -> T) -> T {
        self.start_timer(name);

        let rv = closure();

        self.stop_timer(name);
        
        rv
    }

    pub fn start_timer(&mut self, name: &str) {
        if !self.enabled {
            return;
        }

        self.live_timers.start(name);
    }

    pub fn stop_timer(&mut self, name: &str) {
        if !self.enabled {
            return;
        }

        let timing = self.live_timers.stop(name);
        self.metrics.push(Metric::Timing(timing));
    }

    pub fn flush_metrics(&mut self) {
        if !self.enabled {
            return;
        }

        // package up all our data into a metrics object
        let metrics = self.metrics.clone();

        // transmit the metrics
        self.met_tx.send(metrics).unwrap();

        // clear our counters
        self.metrics.clear();
    }
}
