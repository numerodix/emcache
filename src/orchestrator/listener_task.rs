use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

use options::MemcacheOptions;

use super::DriverTask;
use super::MetricsTask;
use super::TransportId;
use super::TransportTask;


pub struct ListenerTask {
    cur_transport_id: TransportId,
    options: MemcacheOptions,
}

impl ListenerTask {
    pub fn new(options: MemcacheOptions) -> ListenerTask {
        ListenerTask {
            cur_transport_id: 0,
            options: options,
        }
    }

    fn next_transport_id(&mut self) -> TransportId {
        let id = self.cur_transport_id.clone();
        self.cur_transport_id += 1;
        id
    }

    pub fn run(&mut self) {
        // Initialize the metrics sink
        let (met_tx, met_rx) = mpsc::channel();
        let metrics = MetricsTask::new(met_rx);

        thread::spawn(move || {
            metrics.run();
        });

        // Initialize the driver
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let driver = DriverTask::new(cmd_rx,
                                     met_tx.clone(),
                                     self.options.clone());

        thread::spawn(move || {
            driver.run();
        });

        // Start up a tcp server
        let (host, port) = self.options.get_bind_params();
        let tcp_listener = TcpListener::bind(("localhost", port)).unwrap();

        for stream in tcp_listener.incoming() {
            match stream {
                Ok(stream) => {
                    let id = self.next_transport_id();
                    let cmd_tx = cmd_tx.clone();
                    let met_tx = met_tx.clone();
                    let opts = self.options.clone();
                    let task = TransportTask::new(id, cmd_tx, met_tx, opts);

                    thread::spawn(move || {
                        task.run(stream);
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
