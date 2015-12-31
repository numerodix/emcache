use metrics::Metric;
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
            for metric in metrics.metrics {
                match metric {
                    Metric::Timing(timing) => {
                        ts.add_timing(&timing);
                    }
                }
            }

            // Is is time to print a summary?
            if last_summary_at + self.summary_interval < time_now() {
                self.print_summary(&ts);
                ts.clear();

                last_summary_at = time_now();
            }
        }
    }

    pub fn print_summary(&self, ts: &TimeSeries) {
        let agg_mets = ts.aggregate_metrics();
        let mut names: Vec<&String> = agg_mets.keys().collect();
        names.sort();

        println!("== Metrics {}s snapshot at {} ==",
                 self.summary_interval,
                 time_now() as u64);

        for name in names {
            let agg = agg_mets.get(name).unwrap();

            let avg = agg.avg.unwrap_or(-1.0) * 1000.0;
            let p0 = agg.p0.unwrap_or(-1.0) * 1000.0;
            let p99 = agg.p99.unwrap_or(-1.0) * 1000.0;
            let p999 = agg.p999.unwrap_or(-1.0) * 1000.0;

            println!("{:30}  avg: {:.3}ms  p0: {:.3}ms  p99: {:.3}ms  p99.9: {:.3}ms",
                     name,
                     avg,
                     p0,
                     p99,
                     p999);
        }
    }
}
