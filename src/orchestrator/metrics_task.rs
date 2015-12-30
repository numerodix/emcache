use super::MetricsReceiver;


pub struct MetricsTask {
    met_rx: MetricsReceiver,
}

impl MetricsTask {
    pub fn new(met_rx: MetricsReceiver) -> MetricsTask {
        MetricsTask { met_rx: met_rx }
    }

    pub fn run(&self) {
        loop {
            // Receive metrics
            let metrics = self.met_rx.recv().unwrap();
            println!("MetricsTask received: {:?}", metrics);
        }
    }
}
