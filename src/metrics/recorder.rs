use std::collections::HashMap;

use metrics::Metrics;
use platform::time::time_now;
use orchestrator::MetricsSender;


pub struct MetricsRecorder {
    // name, start_time
    live_timers: HashMap<String, f64>,
    // name, duration
    done_timers: HashMap<String, f64>,

    met_tx: MetricsSender,
}

impl MetricsRecorder {
    pub fn new(met_tx: MetricsSender) -> MetricsRecorder {
        MetricsRecorder {
            live_timers: HashMap::new(),
            done_timers: HashMap::new(),
            met_tx: met_tx,
        }
    }

    pub fn start_timer(&mut self, name: &str) {
        //println!("Timed starting: {:?}", name);
        self.live_timers.insert(name.to_string(), time_now());
    }

    pub fn stop_timer(&mut self, name: &str) {
        //println!("Timed stopping: {:?}", name);
        let stop_time = time_now();

        let opt = self.live_timers.remove(name);
        if opt.is_none() {
            panic!("Tried to stop non-live timer: {:?}", name);
        }

        let start_time = opt.unwrap();
        let duration = stop_time - start_time;

        self.done_timers.insert(name.to_string(), duration);
        //println!("Timed {:20}: {:?}s", name, duration);
    }

    pub fn flush_metrics(&mut self) {
        let mut metrics = Metrics::new();
        metrics.with_timers(self.done_timers.clone());

        self.met_tx.send(metrics);

        self.done_timers.clear();
    }
}
