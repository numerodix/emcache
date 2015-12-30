use std::collections::HashMap;

use metrics::MetricsRecorder;
use metrics::Timer;
use protocol::Driver;
use storage::Cache;
use tcp_transport::stats::TransportStats;

use super::CmdReceiver;
use super::MetricsSender;
use super::TransportId;


type StatsMap = HashMap<TransportId, TransportStats>;

fn compute_stats_sums(map: &StatsMap) -> TransportStats {
    let mut total_stats = TransportStats::new();

    for (_, value) in map {
        total_stats.bytes_read += value.bytes_read;
        total_stats.bytes_written += value.bytes_written;
    }

    total_stats
}


pub struct DriverTask {
    cmd_rx: CmdReceiver,
    met_tx: MetricsSender,
}

impl DriverTask {
    pub fn new(cmd_rx: CmdReceiver, met_tx: MetricsSender) -> DriverTask {
        DriverTask {
            cmd_rx: cmd_rx,
            met_tx: met_tx,
        }
    }

    pub fn run(&self) {
        let cache = Cache::new(64 * 1024 * 1024); // 64mb
        let mut driver = Driver::new(cache);

        // Here we store stats per transport
        let mut transport_stats: StatsMap = HashMap::new();

        // For collecting server metrics
        let mut rec = MetricsRecorder::new(self.met_tx.clone());

        loop {
            // Time the whole loop
            rec.start_timer("DriverTask:loop");

            // Receive command
            let (id, resp_tx, cmd, stats) = {
                let _t = Timer::new(&mut rec, "DriverTask:recv_cmd");
                self.cmd_rx.recv().unwrap()
            };

            // Update our stats store
            transport_stats.insert(id, stats);

            // Update the driver's view of all transport metrics
            let total_stats = compute_stats_sums(&transport_stats);
            driver.update_transport_stats(total_stats);

            // Execute the command
            let resp = {
                let _t = Timer::new(&mut rec, "DriverTask:exec_cmd");
                driver.run(cmd)
            };

            // Send response
            {
                let _t = Timer::new(&mut rec, "DriverTask:send_resp");
                resp_tx.send(resp).unwrap();
            }

            // Stop timing the loop
            rec.stop_timer("DriverTask:loop");

            // Now flush metrics outside the request path
            rec.flush_metrics();
        }
    }
}
