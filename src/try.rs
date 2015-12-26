use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::sync::mpsc;
use std::thread;

use platform::time::sleep_secs;


fn transport(id: u64, sender: Sender<u64>) {
    for i in 0..10 {
        println!("[transport {}] Sending: {:?}", id, i);
        sender.send(i).unwrap();
        println!("[transport {}] Sent: {:?}", id, i);
        sleep_secs(1.0);
    }
    println!("[transport {}] Stopping", id);
}

fn driver(receiver: Receiver<u64>) {
    loop {
        println!("[driver]      Receiving...");
        let val = receiver.recv().unwrap();
        println!("[driver]      Received: {:?}", val);
    }
}

pub fn run_it() {
    let (tx, rx) = mpsc::channel();

    println!("[listen]      Launching driver");
    thread::spawn(move || {
        driver(rx);
    });

    for i in 1..5 {
        println!("[listen]      Launching transport {}", i);

        let tx_clone = tx.clone();
        thread::spawn(move || {
            transport(i, tx_clone);
        });
    }

    loop {
        sleep_secs(1.0);
    }
}
