use protocol::Driver;
use storage::Cache;

use super::CmdReceiver;


pub struct DriverTask {
    cmd_rx: CmdReceiver,
}

impl DriverTask {
    pub fn new(cmd_rx: CmdReceiver) -> DriverTask {
        DriverTask { cmd_rx: cmd_rx }
    }

    pub fn run(&self) {
        let cache = Cache::new(1024);
        let mut driver = Driver::new(cache);

        loop {
            // Receive command
            let (id, resp_tx, cmd) = self.cmd_rx.recv().unwrap();
            println!("Driver received from {:?}: {:?}", id, cmd);

            // Execute the command
            let resp = driver.run(cmd);

            // Send response
            println!("Driver sending to {:?}: {:?}", id, &resp);
            resp_tx.send(resp).unwrap();
        }
    }
}
