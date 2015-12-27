use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

use super::DriverTask;
use super::TransportTask;
use super::TransportId;


pub struct ListenerTask {
    cur_transport_id: TransportId,
}

impl ListenerTask {
    pub fn new() -> ListenerTask {
        ListenerTask { cur_transport_id: 0 }
    }

    fn next_transport_id(&mut self) -> TransportId {
        let id = self.cur_transport_id.clone();
        self.cur_transport_id += 1;
        id
    }

    pub fn run(&mut self) {
        // Initialize the driver
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let driver_task = DriverTask::new(cmd_rx);

        thread::spawn(move || {
            driver_task.run();
        });

        // Start up a tcp server
        let tcp_listener = TcpListener::bind("127.0.0.1:11311").unwrap();

        for stream in tcp_listener.incoming() {
            match stream {
                Ok(stream) => {
                    let id = self.next_transport_id();
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
