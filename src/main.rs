extern crate linked_hash_map;
extern crate libc;
extern crate time;

mod orchestrator;
mod platform;
mod protocol;
mod storage;
mod tcp_transport;
mod tcp_server;

mod try;

use orchestrator::ListenerTask;
use try::run_it;
use tcp_server::serve_forever;


fn main() {
    let listener_task = ListenerTask::new(4);
    listener_task.run();
    return;


    println!("Launching tcp server...");
    serve_forever();
}
