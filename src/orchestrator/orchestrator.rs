use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::sync::mpsc;
use std::thread;

use platform::time::sleep_secs;
use protocol::Driver;
use protocol::cmd::Cmd;
use protocol::cmd::Get;
use protocol::cmd::Resp;
use protocol::cmd::Value;
use storage::Cache;


type CmdSender = Sender<(u64, Cmd)>;
type CmdReceiver = Receiver<(u64, Cmd)>;

type RespSender = Sender<Resp>;
type RespReceiver = Receiver<Resp>;

type TransportId = u64;

type RespSenders = HashMap<TransportId, RespSender>;
type RespReceivers = HashMap<TransportId, RespReceiver>;


struct DriverTask {
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


struct TransportTask {
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
