use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

use super::CmdReceiver;
use super::CmdSender;
use super::DriverTask;
use super::TransportId;
use super::TransportTask;


pub struct ListenerTask {
    cnt_transports: u64,
}

impl ListenerTask {
    pub fn new() -> ListenerTask {
        ListenerTask {
            cnt_transports: 0,
        }
    }

    pub fn create_cmd_channel(&self) -> (CmdSender, CmdReceiver) {
        mpsc::channel()
    }

    pub fn create_driver(&self,
                         cmd_rx: CmdReceiver)
                         -> DriverTask {
        DriverTask::new(cmd_rx)
    }

    pub fn launch_transport(&self,
                            stream: TcpStream,
                            id: TransportId,
                            cmd_tx: CmdSender) {
        let transport_task = TransportTask::new(id, cmd_tx);

        thread::spawn(move || {
            transport_task.run(stream);
        });
    }

    pub fn run(&mut self) {
        // Init
        let (cmd_tx, cmd_rx) = self.create_cmd_channel();

        let driver_task = self.create_driver(cmd_rx);
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

                    self.launch_transport(stream, id, cmd_tx);
                }
                Err(_) => {
                    println!("Connection failed :(");
                }
            }
        }

        drop(tcp_listener);
    }
}
