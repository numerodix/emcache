use std::net::TcpStream;
use std::io::Read;
use std::io::Write;

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


    fn read_line(&mut self) -> TcpTransportResult<Vec<u8>> {
        // TODO take a limit argument, don't read forever
        let mut byte = [0; 1];
        let mut line = vec![];

        loop {
            match self.stream.read(&mut byte) {
                Ok(_) => {
                    line.push(byte[0]);

                    // Did we find \r\n? Then we've read a whole line
                    if line.ends_with(&[13, 10]) {
                        // pop off the last two chars
                        line.pop();
                        line.pop();

                        break;
                    }
                }
                Err(_) => {
                    return Err(TcpTransportError::SocketReadError);
                }
            }
        }

        Ok(line)
    }


    pub fn read_cmd(&mut self) -> TcpTransportResult<Cmd> {
        let fst_line = self.read_line().unwrap();  // XXX error handling
        let fst_line_str = String::from_utf8(fst_line).unwrap(); // XXX errors

        if fst_line_str == "stats" {
            return Ok(Cmd::Stats);
        }

        Ok(Cmd::Stats)
    }

    pub fn write_resp(&mut self, resp: &Resp) -> TcpTransportResult<()> {
        Ok(())
    }
}
