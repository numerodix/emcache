use protocol::Driver;
use protocol::cmd::Cmd;
use protocol::cmd::Resp;

use super::CmdReceiver;
use super::RespSenders;


pub struct DriverTask {
    driver: Driver,

    cmd_rx: CmdReceiver,
    resp_txs: RespSenders,
}

impl DriverTask {
    pub fn new(driver: Driver,
               cmd_rx: CmdReceiver,
               resp_txs: RespSenders)
               -> DriverTask {
        DriverTask {
            cmd_rx: cmd_rx,
            driver: driver,
            resp_txs: resp_txs,
        }
    }

    pub fn run(&self) {
        loop {
            let (id, cmd) = self.cmd_rx.recv().unwrap();
            println!("Driver received from {:?}: {:?}", id, cmd);

            let resp_tx = self.resp_txs.get(&id).unwrap();
            resp_tx.send(Resp::Error);
        }
    }
}
