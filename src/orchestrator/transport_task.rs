use std::net::TcpStream;
use std::sync::mpsc;

use metrics::MetricsRecorder;
use metrics::Timer;
use options::MemcacheOptions;
use protocol::cmd::Resp;
use tcp_transport::TcpTransport;

use super::CmdSender;
use super::MetricsSender;
use super::RespReceiver;
use super::RespSender;
use super::TransportId;


pub struct TransportTask {
    id: TransportId,
    cmd_tx: CmdSender,
    met_tx: MetricsSender,
    options: MemcacheOptions,
}

impl TransportTask {
    pub fn new(id: TransportId,
               cmd_tx: CmdSender,
               met_tx: MetricsSender,
               options: MemcacheOptions)
               -> TransportTask {
        TransportTask {
            id: id,
            cmd_tx: cmd_tx,
            met_tx: met_tx,
            options: options,
        }
    }

    pub fn run(&self, stream: TcpStream) {
        let mut rec = MetricsRecorder::new(self.met_tx.clone(),
                                           self.options.get_metrics_enabled());

        let mut transport = TcpTransport::new(stream);
        let (resp_tx, resp_rx): (RespSender, RespReceiver) = mpsc::channel();

        loop {
            // Time the whole loop
            rec.start_timer("TransportTask:loop");

            // println!("Ready to read command...");
            let rv = {
                let _t = Timer::new(&mut rec, "TransportTask:read_cmd");
                transport.read_cmd()
            };

            // If we couldn't parse the command return an error
            if !rv.is_ok() {
                println!("Failed to read command, returning error");
                let _ = transport.write_resp(&Resp::Error);
                return; // Here we just drop the connection
            }

            // Send the command to the driver
            let cmd = rv.unwrap();
            let resp_tx_clone = resp_tx.clone();
            let stats = transport.get_stats_clone();
            {
                let _t = Timer::new(&mut rec, "TransportTask:send_cmd");
                self.cmd_tx
                    .send((self.id, resp_tx_clone, cmd, stats))
                    .unwrap();
            }

            // Obtain a response
            let resp = {
                let _t = Timer::new(&mut rec, "TransportTask:recv_resp");
                resp_rx.recv().unwrap()
            };

            // Return a response
            // println!("Returning response: {:?}", &resp);
            let rv = {
                let _t = Timer::new(&mut rec, "TransportTask:write_resp");
                transport.write_resp(&resp)
            };
            if !rv.is_ok() {
                println!("Failed to write response :(");
            }

            // Stop timing the loop
            rec.stop_timer("TransportTask:loop");

            // Now flush metrics outside the request path
            rec.flush_metrics();
        }
    }
}
