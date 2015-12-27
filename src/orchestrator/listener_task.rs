use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use platform::time::sleep_secs;
use protocol::Driver;
use protocol::cmd::Cmd;
use protocol::cmd::Get;
use protocol::cmd::Resp;
use protocol::cmd::Value;
use storage::Cache;

use super::DriverTask;
use super::TransportTask;


pub struct ListenerTask {
    max_transports: u64,
}

impl ListenerTask {
    pub fn new(max_transports: u64) -> ListenerTask {
        ListenerTask { max_transports: max_transports }
    }

    pub fn run(&self) {
        let cache = Cache::new(1024);
        let mut driver = Driver::new(cache);

        let (cmd_tx, cmd_rx) = mpsc::channel();

        let mut resp_txs = HashMap::new();
        let mut resp_rxs = HashMap::new();

        for i in 0..self.max_transports {
            let (resp_tx, resp_rx) = mpsc::channel();
            resp_txs.insert(i, resp_tx);
            resp_rxs.insert(i, resp_rx);
        }

        let driver_task = DriverTask::new(driver, cmd_rx, resp_txs);
        thread::spawn(move || {
            driver_task.run();
        });

        for id in 0..2 {
            let resp_rx = resp_rxs.remove(&id).unwrap();
            let transport_task = TransportTask::new(id,
                                                    cmd_tx.clone(),
                                                    resp_rx);

            thread::spawn(move || {
                transport_task.run();
            });
        }

        loop {
            sleep_secs(1.0);
        }
    }
}
