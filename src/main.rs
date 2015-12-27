extern crate linked_hash_map;
extern crate libc;
extern crate time;

mod orchestrator;
mod platform;
mod protocol;
mod storage;
mod tcp_transport;

use orchestrator::ListenerTask;


fn main() {
    let mut listener_task = ListenerTask::new(4096);

    println!("Launching tcp server...");
    listener_task.run();
}
