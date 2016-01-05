// Benchmark testing primitives
#![feature(test)]
extern crate test;

#[macro_use]
extern crate maplit;
extern crate bufstream;
extern crate docopt;
extern crate linked_hash_map;
extern crate libc;
extern crate net2;
//extern crate rand;
extern crate rustc_serialize;
extern crate time;

mod common;
mod metrics;
mod options;
mod orchestrator;
mod platform;
mod protocol;
mod storage;
mod tcp_transport;
mod testlib;

use common::consts;
use options::parse_args;
use orchestrator::ListenerTask;


fn print_version() {
    println!("{} {}", consts::APP_NAME, consts::APP_VERSION);
}

fn main() {
    print_version();

    let opts = parse_args();
    if opts.flag_version {
        // We're done here :)
        return;
    }

    println!("Running tcp server on {} with {}mb capacity...",
             opts.get_bind_string(),
             opts.get_mem_limit());
    let mut listener_task = ListenerTask::new(opts.clone());
    listener_task.run();
}
