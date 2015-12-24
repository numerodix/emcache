use std::net::TcpStream;

use protocol::cmd::Cmd;
use protocol::cmd::Resp;

use super::errors::TcpTransportError;
use super::typedefs::TcpTransportResult;


struct TcpTransport {
    stream: TcpStream,
}

impl TcpTransport {
    pub fn new(mut stream: TcpStream) -> TcpTransport {
        TcpTransport { stream: stream }
    }


    pub fn read_cmd(&mut self) -> TcpTransportResult<Cmd> {
        Ok(Cmd::Stats)
    }

    pub fn write_resp(&mut self, resp: &Resp) -> TcpTransportResult<()> {
        Ok(())
    }
}
