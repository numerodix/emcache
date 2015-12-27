use platform::time::sleep_secs;
use protocol::cmd::Cmd;
use protocol::cmd::Resp;

use super::CmdSender;
use super::RespReceiver;
use super::TransportId;


pub struct TransportTask {
    id: TransportId,
    cmd_tx: CmdSender,
    resp_rx: RespReceiver,
}

impl TransportTask {
    pub fn new(id: TransportId,
               cmd_tx: CmdSender,
               resp_rx: RespReceiver)
               -> TransportTask {
        TransportTask {
            id: id,
            cmd_tx: cmd_tx,
            resp_rx: resp_rx,
        }
    }

    pub fn run(&self) {
        loop {
            self.cmd_tx.send((self.id, Cmd::Stats)).unwrap();
            let val = self.resp_rx.recv().unwrap();
            println!("Transport {:?} received: {:?}", self.id, val);

            sleep_secs(1.0);
        }
    }
}
