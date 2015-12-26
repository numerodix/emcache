extern crate linked_hash_map;
extern crate libc;
extern crate time;

mod platform;
mod protocol;
mod storage;
mod tcp_transport;
mod tcp_server;

use tcp_server::serve_forever;


fn main() {
    println!("Launching tcp server...");
    serve_forever();
}
