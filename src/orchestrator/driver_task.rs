use std::collections::HashMap;

use protocol::Driver;
use storage::Cache;
use tcp_transport::stats::TransportStats;

use super::CmdReceiver;
use super::TransportId;


type StatsMap = HashMap<TransportId, TransportStats>;

fn compute_stats_sums(map: &StatsMap) -> TransportStats {
    let mut total_stats = TransportStats::new();

    for (key, value) in map {
        total_stats.bytes_read += value.bytes_read;
        total_stats.bytes_written += value.bytes_written;
    }

    total_stats
}


pub struct DriverTask {
    cmd_rx: CmdReceiver,
}

impl DriverTask {
    pub fn new(cmd_rx: CmdReceiver) -> DriverTask {
        DriverTask { cmd_rx: cmd_rx }
    }

    pub fn run(&self) {
        let cache = Cache::new(64 * 1024 * 1024); // 64mb
        let mut driver = Driver::new(cache);

        // Here we store metrics per transport
        let mut transport_stats: StatsMap = HashMap::new();

        loop {
            // Receive command
            let (id, resp_tx, cmd, stats) = self.cmd_rx.recv().unwrap();
            println!("Driver received from {:?}: {:?}", id, cmd);

            // Update our stats store
            transport_stats.insert(id, stats);

            // Update the driver's view of all transport metrics
            let total_stats = compute_stats_sums(&transport_stats);
            driver.update_transport_stats(total_stats);

            // Execute the command
            let resp = driver.run(cmd);

            // Send response
            println!("Driver sending to {:?}: {:?}", id, &resp);
            resp_tx.send(resp).unwrap();
        }
    }
}
