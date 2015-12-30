use metrics::TimeSeries;
use platform::time::time_now;

use super::MetricsReceiver;


pub struct MetricsTask {
    met_rx: MetricsReceiver,

    summary_interval: f64,
}

impl MetricsTask {
    pub fn new(met_rx: MetricsReceiver) -> MetricsTask {
        MetricsTask {
            met_rx: met_rx,

            summary_interval: 1.0,
        }
    }

    pub fn run(&self) {
        let mut ts = TimeSeries::new();
        let mut last_summary_at = time_now();

        loop {
            // Receive metrics
            let metrics = self.met_rx.recv().unwrap();
            ts.merge(&metrics.timers);

            // Is is time to print a summary?
            if last_summary_at + self.summary_interval < time_now() {
                self.print_summary(&ts);
                ts.clear();

                last_summary_at = time_now();
            }
        }
    }

    pub fn print_summary(&self, ts: &TimeSeries) {
        let comp_mets = ts.compute_metrics();
        let mut names: Vec<&String> = comp_mets.keys().collect();
        names.sort();

        println!("== Metrics {}s snapshot at {} ==",
                 self.summary_interval,
                 time_now() as u64);

        for name in names {
            let comp = comp_mets.get(name).unwrap();

            let avg = comp.avg.unwrap_or(-1.0) * 1000.0;
            let p90 = comp.p90.unwrap_or(-1.0) * 1000.0;
            let p99 = comp.p99.unwrap_or(-1.0) * 1000.0;
            let p999 = comp.p999.unwrap_or(-1.0) * 1000.0;

            println!("{:30}  avg: {:.3}ms  p90: {:.3}ms  p99: {:.3}ms  \
                      p99.9: {:.3}ms",
                     name,
                     avg,
                     p90,
                     p99,
                     p999);
        }
    }
}
