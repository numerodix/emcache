use std::net::TcpStream;

use platform::time::sleep_secs;
use protocol::cmd::Cmd;
use protocol::cmd::Resp;
use tcp_transport::TcpTransport;

use super::CmdSender;
use super::RespReceiver;
use super::TransportId;


pub struct TransportTask {
    id: TransportId,
    cmd_tx: CmdSender,
    resp_rx: RespReceiver,
}

impl TransportTask {
    pub fn new(id: TransportId,
               cmd_tx: CmdSender,
               resp_rx: RespReceiver)
               -> TransportTask {
        TransportTask {
            id: id,
            cmd_tx: cmd_tx,
            resp_rx: resp_rx,
        }
    }

    pub fn run(&self, stream: TcpStream) {
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

            // cmd -> resp
            let cmd = rv.unwrap();
            self.cmd_tx.send((self.id, cmd)).unwrap();
            let resp = self.resp_rx.recv().unwrap();

            // Return a response
            println!("Returning response: {:?}", &resp);
            let rv = transport.write_resp(&resp);
            if !rv.is_ok() {
                println!("Failed to write response :(");
            }
        }
    }
}
