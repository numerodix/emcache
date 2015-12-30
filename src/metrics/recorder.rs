use metrics::Metrics;
use orchestrator::MetricsSender;

use super::LiveTimers;
use super::TimeSeries;


pub struct MetricsRecorder {
    live_timers: LiveTimers,
    done_timers: TimeSeries,

    met_tx: MetricsSender,
}

impl MetricsRecorder {
    pub fn new(met_tx: MetricsSender) -> MetricsRecorder {
        MetricsRecorder {
            live_timers: LiveTimers::new(),
            done_timers: TimeSeries::new(),
            met_tx: met_tx,
        }
    }

    pub fn start_timer(&mut self, name: &str) {
        self.live_timers.start(name);
    }

    pub fn stop_timer(&mut self, name: &str) {
        let (start_time, dur) = self.live_timers.stop(name);
        self.done_timers.add_timer(name, start_time, dur);
    }

    pub fn flush_metrics(&mut self) {
        // package up all our data into a metrics object
        let mut metrics = Metrics::new();
        metrics.with_timers(self.done_timers.clone());

        // transmit the metrics
        self.met_tx.send(metrics).unwrap();

        // clear our counters
        self.done_timers.clear();
    }
}
