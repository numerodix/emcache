use std::collections::HashMap;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

use platform::time::sleep_secs;
use protocol::Driver;
use protocol::cmd::Cmd;
use protocol::cmd::Get;
use protocol::cmd::Resp;
use protocol::cmd::Value;
use storage::Cache;

use super::CmdReceiver;
use super::CmdSender;
use super::DriverTask;
use super::RespReceiver;
use super::RespReceivers;
use super::RespSenders;
use super::TransportId;
use super::TransportTask;


pub struct ListenerTask {
    cnt_transports: u64,
    max_transports: u64,
}

impl ListenerTask {
    pub fn new(max_transports: u64) -> ListenerTask {
        ListenerTask {
            cnt_transports: 0,
            max_transports: max_transports,
        }
    }

    pub fn create_cmd_channel(&self) -> (CmdSender, CmdReceiver) {
        mpsc::channel()
    }

    pub fn create_resp_channels(&self) -> (RespSenders, RespReceivers) {
        let mut resp_txs = HashMap::new();
        let mut resp_rxs = HashMap::new();

        for i in 0..self.max_transports {
            let (resp_tx, resp_rx) = mpsc::channel();

            resp_txs.insert(i, resp_tx);
            resp_rxs.insert(i, resp_rx);
        }

        (resp_txs, resp_rxs)
    }

    pub fn create_driver(&self,
                         cmd_rx: CmdReceiver,
                         resp_txs: RespSenders)
                         -> DriverTask {
        let cache = Cache::new(1024);
        let mut driver = Driver::new(cache);

        DriverTask::new(driver, cmd_rx, resp_txs)
    }

    pub fn create_transport(&self,
                            id: TransportId,
                            cmd_tx: CmdSender,
                            resp_rx: RespReceiver)
                            -> TransportTask {
        TransportTask::new(id, cmd_tx, resp_rx)
    }

    pub fn launch_transport(&self,
                            stream: TcpStream,
                            id: TransportId,
                            cmd_tx: CmdSender,
                            resp_rx: RespReceiver) {
        let transport_task = self.create_transport(id, cmd_tx, resp_rx);

        thread::spawn(move || {
            transport_task.run();
        });
    }

    pub fn run(&mut self) {
        // Init
        let (cmd_tx, cmd_rx) = self.create_cmd_channel();
        let (mut resp_txs, mut resp_rxs) = self.create_resp_channels();

        let driver_task = self.create_driver(cmd_rx, resp_txs);
        thread::spawn(move || {
            driver_task.run();
        });

        // Start up a server
        let tcp_listener = TcpListener::bind("127.0.0.1:11311").unwrap();

        for stream in tcp_listener.incoming() {
            match stream {
                Ok(stream) => {
                    // allocate transport id
                    let id = self.cnt_transports.clone();
                    self.cnt_transports += 1;

                    let cmd_tx = cmd_tx.clone();
                    let resp_rx = resp_rxs.remove(&id).unwrap();

                    self.launch_transport(stream, id, cmd_tx, resp_rx);
                }
                Err(_) => {
                    println!("Connection failed :(");
                }
            }
        }

        drop(tcp_listener);
    }
}
