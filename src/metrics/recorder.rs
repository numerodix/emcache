use orchestrator::MetricsSender;

use super::LiveTimers;
use super::Metric;
use super::Metrics;


pub struct MetricsRecorder {
    live_timers: LiveTimers,
    metrics: Metrics,

    met_tx: MetricsSender,
}

impl MetricsRecorder {
    pub fn new(met_tx: MetricsSender) -> MetricsRecorder {
        MetricsRecorder {
            live_timers: LiveTimers::new(),
            metrics: Metrics::new(),
            met_tx: met_tx,
        }
    }

    pub fn start_timer(&mut self, name: &str) {
        self.live_timers.start(name);
    }

    pub fn stop_timer(&mut self, name: &str) {
        let timing = self.live_timers.stop(name);
        self.metrics.push(Metric::Timing(timing));
    }

    pub fn flush_metrics(&mut self) {
        // package up all our data into a metrics object
        let mut metrics = self.metrics.clone();

        // transmit the metrics
        self.met_tx.send(metrics).unwrap();

        // clear our counters
        self.metrics.clear();
    }
}
