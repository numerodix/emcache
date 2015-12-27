use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

use super::DriverTask;
use super::TransportTask;


pub struct ListenerTask {
    cnt_transports: u64,
}

impl ListenerTask {
    pub fn new() -> ListenerTask {
        ListenerTask { cnt_transports: 0 }
    }

    pub fn run(&mut self) {
        // Init
        let (cmd_tx, cmd_rx) = mpsc::channel();

        let driver_task = DriverTask::new(cmd_rx);
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
                    let transport_task = TransportTask::new(id, cmd_tx);

                    thread::spawn(move || {
                        transport_task.run(stream);
                    });

                }
                Err(_) => {
                    println!("Connection failed :(");
                }
            }
        }

        drop(tcp_listener);
    }
}
