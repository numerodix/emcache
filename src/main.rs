extern crate time;

mod protocol;
mod storage;
mod tcp_transport;
mod tcp_server;

mod try;
mod try_refs;

use try_refs::must;
use tcp_server::serve_forever;


fn main() {
    must();

    println!("Launching tcp server...");
    //serve_forever();
}
