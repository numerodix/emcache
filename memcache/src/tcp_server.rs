use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

use tcp_transport::TcpTransport;


fn handle_client_playground(mut stream: TcpStream) {
    let mut transport = TcpTransport::new(stream);

    loop {
        let rv = transport.read_cmd();
        match rv {
            Ok(cmd) => {
                println!("Read cmd: {:?}", cmd);
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}


fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 1];

    loop {
        let bytelen = stream.read(&mut buf).unwrap();
        println!("Client sent    : {:?}", buf);

        println!("Responding with: {:?}", buf);
        let bytelen = stream.write(&buf);

        println!("");
    }
}

pub fn listen() {
    let listener = TcpListener::bind("127.0.0.1:11311").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client_playground(stream));
            }
            Err(e) => {
                println!("Connection failed :(");
            }
        }
    }

    drop(listener);
}
