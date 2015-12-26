use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::sync::mpsc;
use std::thread;

use platform::time::sleep_secs;
use protocol::cmd::Cmd;
use protocol::cmd::Get;
use protocol::cmd::Resp;
use protocol::cmd::Value;


type CmdSender = Sender<Cmd>;
type CmdReceiver = Receiver<Cmd>;

type RespSender = Sender<Resp>;
type RespReceiver = Receiver<Resp>;


fn transport(id: u64, sender: CmdSender, receiver: RespReceiver) {
    for i in 0..5 {
        let val = Cmd::Get(Get::new("x"));
        println!("[transport {}] Sending: {:?}", id, val.clone());
        sender.send(val.clone()).unwrap();
        println!("[transport {}] Sent: {:?}", id, val.clone());

        println!("[transport {}] Receiving...", id);
        let val = receiver.recv().unwrap();
        println!("[transport {}] Received: {:?}", id, val);

        sleep_secs(1.0);
    }

    println!("[transport {}] Stopping", id);
}

fn driver(sender: RespSender, receiver: CmdReceiver) {
    loop {
        println!("[driver]      Receiving...");
        let val = receiver.recv().unwrap();
        println!("[driver]      Received: {:?}", val);

        let val = Resp::Value(Value::new("x", vec![1, 2, 3]));
        println!("[driver]      Sending: {:?}", val.clone());
        sender.send(val.clone()).unwrap();
        println!("[driver]      Sent: {:?}", val.clone());
    }

    println!("[driver]      Stopping");
}

pub fn run_it() {
    let (c_tx, c_rx) = mpsc::channel();
    let (r_tx, r_rx) = mpsc::channel();

    println!("[listen]      Launching driver");
    thread::spawn(move || {
        driver(r_tx, c_rx);
    });

    println!("[listen]      Launching transport {}", 1);
    thread::spawn(move || {
        transport(1, c_tx, r_rx);
    });

    loop {
        sleep_secs(1.0);
    }
}
