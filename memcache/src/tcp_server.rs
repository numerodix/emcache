use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

use protocol::Driver;
use protocol::cmd::Resp;
use storage::Cache;
use tcp_transport::TcpTransport;


fn simple_memcache(stream: TcpStream) {
    let cache = Cache::new(1024);
    let mut driver = Driver::new(cache);

    let mut transport = TcpTransport::new(stream);


    loop {
        let rv = transport.read_cmd();

        // If we couldn't parse the command return an error
        if !rv.is_ok() {
            transport.write_resp(&Resp::Error).unwrap();
            continue;
        }

        // Execute the command
        let cmd = rv.unwrap();
        println!("Received command  : {:?}", cmd);
        let resp = driver.run(cmd);
        println!("Returning response: {:?}", resp);

        // Return a response
        let rv = transport.write_resp(&resp);
        if !rv.is_ok() {
            println!("Failed to write response :(");
        }
    }
}


fn handle_client_playground(stream: TcpStream) {
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
                thread::spawn(move || simple_memcache(stream));
            }
            Err(_) => {
                println!("Connection failed :(");
            }
        }
    }

    drop(listener);
}
