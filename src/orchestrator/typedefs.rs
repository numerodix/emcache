use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

use protocol::cmd::Cmd;
use protocol::cmd::Resp;


// Only used for display purposes
pub type TransportId = u64;

pub type RespSender = Sender<Resp>;
pub type RespReceiver = Receiver<Resp>;

pub type CmdSender = Sender<(TransportId, RespSender, Cmd)>;
pub type CmdReceiver = Receiver<(TransportId, RespSender, Cmd)>;
