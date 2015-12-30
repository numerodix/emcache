use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

use metrics::Metrics;
use protocol::cmd::Cmd;
use protocol::cmd::Resp;
use tcp_transport::stats::TransportStats;


// Cmd/Resp Protocol

pub type TransportId = u64;

pub type RespSender = Sender<Resp>;
pub type RespReceiver = Receiver<Resp>;

pub type CmdSender = Sender<(TransportId, RespSender, Cmd, TransportStats)>;
pub type CmdReceiver = Receiver<(TransportId,
                                 RespSender,
                                 Cmd,
                                 TransportStats)>;

// Metrics

pub type MetricsSender = Sender<Metrics>;
pub type MetricsReceiver = Receiver<Metrics>;
