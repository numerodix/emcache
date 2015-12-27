use protocol::Driver;
use protocol::cmd::Cmd;
use protocol::cmd::Resp;
use storage::Cache;

use super::CmdReceiver;
use super::RespSenders;


pub struct DriverTask {
    cmd_rx: CmdReceiver,
    resp_txs: RespSenders,
}

impl DriverTask {
    pub fn new(cmd_rx: CmdReceiver, resp_txs: RespSenders) -> DriverTask {
        DriverTask {
            cmd_rx: cmd_rx,
            resp_txs: resp_txs,
        }
    }

    pub fn run(&self) {
        let cache = Cache::new(1024);
        let mut driver = Driver::new(cache);

        loop {
            // Receive command
            let (id, cmd) = self.cmd_rx.recv().unwrap();
            println!("Driver received from {:?}: {:?}", id, cmd);

            // Execute the command
            let resp = driver.run(cmd);

            // Send response
            let resp_tx = self.resp_txs.get(&id).unwrap();
            println!("Driver sending to {:?}: {:?}", id, &resp);
            resp_tx.send(resp);
        }
    }
}
