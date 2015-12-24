use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;


fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 1];

    loop {
        let rv = stream.read(&mut buf).unwrap();
        println!("Client sent    : {:?}", buf);

        println!("Responding with: {:?}", buf);
        let rv = stream.write(&buf);

        println!("");
    }
}

pub fn listen() {
    let listener = TcpListener::bind("127.0.0.1:11311").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Connection failed :(");
            }
        }
    }

    drop(listener);
}
