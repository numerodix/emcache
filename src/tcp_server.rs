use std::net::TcpListener;
use std::net::TcpStream;

use protocol::Driver;
use protocol::cmd::Resp;
use storage::Cache;
use tcp_transport::TcpTransport;


fn handle_client(driver: &mut Driver, stream: TcpStream) {
    let mut transport = TcpTransport::new(stream);

    loop {
        println!("Ready to read command...");
        let rv = transport.read_cmd();

        // If we couldn't parse the command return an error
        if !rv.is_ok() {
            println!("Failed to read command, returning error");
            transport.write_resp(&Resp::Error);
            return; // Here we just drop the connection
        }

        // Execute the command
        let cmd = rv.unwrap();
        println!("Received command  : {:?}", cmd);
        let resp = driver.run(cmd);

        // Return a response
        println!("Returning response: {:?}", resp);
        let rv = transport.write_resp(&resp);
        if !rv.is_ok() {
            println!("Failed to write response :(");
        }
    }
}

pub fn serve_forever() {
    let cache = Cache::new(1024);
    let mut driver = Driver::new(cache);

    let listener = TcpListener::bind("127.0.0.1:11311").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(&mut driver, stream);
            }
            Err(_) => {
                println!("Connection failed :(");
            }
        }
    }

    drop(listener);
}
