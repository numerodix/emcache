use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::sync::mpsc;
use std::thread;

use platform::time::sleep_secs;
use protocol::cmd::Cmd;
use protocol::cmd::Get;
use protocol::cmd::Resp;
use protocol::cmd::Value;


type CmdSender = Sender<(u64, Cmd)>;
type CmdReceiver = Receiver<(u64, Cmd)>;

type RespSender = Sender<Resp>;
type RespReceiver = Receiver<Resp>;


fn transport(id: u64, sender: CmdSender, receiver: RespReceiver) {
    for i in 0..5 {
        let key = format!("x{}", id);
        let val = Cmd::Get(Get { key: key });
        println!("[transport {}] Sending: {:?}", id, val.clone());
        sender.send((id, val.clone())).unwrap();
        println!("[transport {}] Sent: {:?}", id, val.clone());

        println!("[transport {}] Receiving...", id);
        let val = receiver.recv().unwrap();
        println!("[transport {}] Received: {:?}", id, val);

        sleep_secs(1.0);
    }

    println!("[transport {}] Stopping", id);
}

fn driver(receiver: CmdReceiver, sender1: RespSender, sender2: RespSender) {
    loop {
        println!("[driver]      Receiving...");
        let (id, cmd) = receiver.recv().unwrap();
        println!("[driver]      Received: {:?}", cmd);

        let mut key = "?".to_string();
        match cmd {
            Cmd::Get(get) => {
                key = get.key;
            }
            _ => (),
        }

        let val = Resp::Value(Value {
            key: key,
            data: vec![1, 2, 3],
        });
        println!("[driver]      Sending: {:?}", val.clone());
        match id {
            1 => {
                sender1.send(val.clone()).unwrap();
            }
            2 => {
                sender2.send(val.clone()).unwrap();
            }
            _ => (),
        }
        println!("[driver]      Sent: {:?}", val.clone());
    }

    println!("[driver]      Stopping");
}

pub fn run_it() {
    let (c_tx, c_rx) = mpsc::channel();
    let c_tx2 = c_tx.clone();

    let (r_tx1, r_rx1) = mpsc::channel();
    let (r_tx2, r_rx2) = mpsc::channel();

    println!("[listen]      Launching driver");
    thread::spawn(move || {
        driver(c_rx, r_tx1, r_tx2);
    });

    println!("[listen]      Launching transport {}", 1);
    thread::spawn(move || {
        transport(1, c_tx, r_rx1);
    });

    println!("[listen]      Launching transport {}", 2);
    thread::spawn(move || {
        transport(2, c_tx2, r_rx2);
    });

    loop {
        sleep_secs(1.0);
    }
}
