#[macro_use]
extern crate maplit;
extern crate docopt;
extern crate linked_hash_map;
extern crate libc;
extern crate rand;
extern crate rustc_serialize;
extern crate time;

mod metrics;
mod options;
mod orchestrator;
mod platform;
mod protocol;
mod storage;
mod tcp_transport;
mod testlib;

use std::env;

use options::parse_args;
use orchestrator::ListenerTask;


fn main() {
    let opts = parse_args(env::args().collect());

    let mut listener_task = ListenerTask::new(opts.clone());

    println!("Launching tcp server on {} with {}mb capacity...",
             opts.get_bind_string(),
             opts.get_mem_limit());

    listener_task.run();
}
