extern crate time;

mod protocol;
mod storage;
mod tcp_transport;
mod tcp_server;

use tcp_server::serve_forever;


fn main() {
    println!("Launching tcp server...");
    serve_forever();
}
