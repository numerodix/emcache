use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

use protocol::cmd::Cmd;
use protocol::cmd::Resp;


pub type CmdSender = Sender<(u64, Cmd)>;
pub type CmdReceiver = Receiver<(u64, Cmd)>;

pub type RespSender = Sender<Resp>;
pub type RespReceiver = Receiver<Resp>;

pub type TransportId = u64;

pub type RespSenders = HashMap<TransportId, RespSender>;
pub type RespReceivers = HashMap<TransportId, RespReceiver>;
