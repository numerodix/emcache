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


struct DriverTask {
    driver: Driver,

    cmd_rx: CmdReceiver,
    resp_chans: HashMap<TransportId, RespSender>,
}

impl DriverTask {
    pub fn new(driver: Driver, cmd_rx: CmdReceiver) -> DriverTask {
        DriverTask {
            cmd_rx: cmd_rx,
            driver: driver,
            resp_chans: HashMap::new(),
        }
    }

    pub fn add_transport(&mut self, id: u64, chan: RespSender) {
        self.resp_chans.insert(id, chan);
    }

    pub fn run(&self) {
        loop {
            let (id, cmd) = self.cmd_rx.recv().unwrap();

            let resp_tx = self.resp_chans.get(&id).unwrap();
            resp_tx.send(Resp::Error);
        }
    }
}


struct ListenerTask;

impl ListenerTask {
    pub fn new() -> ListenerTask {
        ListenerTask
    }

    pub fn run(&self) {
        let cache = Cache::new(1024);
        let mut driver = Driver::new(cache);

        let (cmd_tx, cmd_rx) = mpsc::channel();

        let driver_task = DriverTask::new(driver, cmd_rx);
        thread::spawn(move || {
            driver_task.run();
        });

        let (resp_tx, resp_rx) = mpsc::channel();
        driver_task.add_transport(1, resp_tx);
    }
}

