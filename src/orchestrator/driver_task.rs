use std::collections::HashMap;

use protocol::Driver;
use storage::Cache;
use tcp_transport::metrics::TransportMetrics;

use super::CmdReceiver;
use super::TransportId;


type MetricsMap = HashMap<TransportId, TransportMetrics>;


pub struct DriverTask {
    cmd_rx: CmdReceiver,
}

impl DriverTask {
    pub fn new(cmd_rx: CmdReceiver) -> DriverTask {
        DriverTask { cmd_rx: cmd_rx }
    }

    // TODO this doesn't seem to belong here
    fn compute_metrics_sums(&self, map: &MetricsMap) -> TransportMetrics {
        let mut total_metrics = TransportMetrics::new();

        for (key, value) in map {
            total_metrics.bytes_read += value.bytes_read;
            total_metrics.bytes_written += value.bytes_written;
        }

        total_metrics
    }

    pub fn run(&self) {
        let cache = Cache::new(1024);
        let mut driver = Driver::new(cache);

        // Here we store metrics per transport
        let mut transport_metrics: MetricsMap = HashMap::new();

        loop {
            // Receive command
            let (id, resp_tx, cmd, metrics) = self.cmd_rx.recv().unwrap();
            println!("Driver received from {:?}: {:?}", id, cmd);

            // Update our metrics store
            transport_metrics.insert(id, metrics);

            // Update the driver's view of all transport metrics
            let total_metrics = self.compute_metrics_sums(&transport_metrics);
            driver.update_transport_metrics(total_metrics);

            // Execute the command
            let resp = driver.run(cmd);

            // Send response
            println!("Driver sending to {:?}: {:?}", id, &resp);
            resp_tx.send(resp).unwrap();
        }
    }
}
