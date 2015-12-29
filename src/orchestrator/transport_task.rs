use std::net::TcpStream;
use std::sync::mpsc;

use metrics::MetricsRecorder;
use metrics::Timer;
use protocol::cmd::Resp;
use tcp_transport::TcpTransport;

use super::CmdSender;
use super::RespReceiver;
use super::RespSender;
use super::TransportId;


pub struct TransportTask {
    id: TransportId,
    cmd_tx: CmdSender,
}

impl TransportTask {
    pub fn new(id: TransportId, cmd_tx: CmdSender) -> TransportTask {
        TransportTask {
            id: id,
            cmd_tx: cmd_tx,
        }
    }

    pub fn run(&self, stream: TcpStream) {
        let mut recorder = MetricsRecorder::new();
        let mut transport = TcpTransport::new(stream);
        let (resp_tx, resp_rx): (RespSender, RespReceiver) = mpsc::channel();

        loop {
            println!("Ready to read command...");
            let rv = {
                Timer::new(&mut recorder, "drop_read_cmd");
                transport.read_cmd()
            };

            // If we couldn't parse the command return an error
            if !rv.is_ok() {
                println!("Failed to read command, returning error");
                transport.write_resp(&Resp::Error);
                return; // Here we just drop the connection
            }

            // Send the command to the driver
            let cmd = rv.unwrap();
            let resp_tx_clone = resp_tx.clone();
            let metrics = transport.get_metrics_clone();
            self.cmd_tx.send((self.id, resp_tx_clone, cmd, metrics)).unwrap();

            // Obtain a response
            let resp = resp_rx.recv().unwrap();

            // Return a response
            println!("Returning response: {:?}", &resp);
            let rv = transport.write_resp(&resp);
            if !rv.is_ok() {
                println!("Failed to write response :(");
            }
        }
    }
}
