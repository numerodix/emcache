use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

use protocol::cmd::Cmd;
use protocol::cmd::Resp;
use tcp_transport::metrics::TransportMetrics;


pub type TransportId = u64;

pub type RespSender = Sender<Resp>;
pub type RespReceiver = Receiver<Resp>;

pub type CmdSender = Sender<(TransportId, RespSender, Cmd, TransportMetrics)>;
pub type CmdReceiver = Receiver<(TransportId,
                                 RespSender,
                                 Cmd,
                                 TransportMetrics)>;
