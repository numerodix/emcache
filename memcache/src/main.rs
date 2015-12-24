extern crate time;

mod protocol;
mod storage;
mod tcp_transport;
mod tcp_server;

use protocol::Driver;

use storage::Cache;
use storage::Key;
use storage::Value;

use tcp_server::listen;


fn main() {
    let mut cache = Cache::new(1024);

    let key = Key::new(vec![1]);
    let value = Value::new(vec![9]);

    cache.set(key.clone(), value.clone()).unwrap();
    println!("Set key {:?} to value {:?}", key.item, value.item);

    let loaded = cache.get(&key).unwrap();
    println!("Loaded key {:?}, got value {:?}", key.item, loaded.item);


    println!("Launching tcp server...");
    listen();
}
