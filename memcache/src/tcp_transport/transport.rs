use std::io::Read;
use std::io::Write;

use protocol::cmd::Cmd;
use protocol::cmd::Resp;

use super::errors::TcpTransportError;
use super::typedefs::TcpTransportResult;


pub struct TcpTransport<T> {
    stream: T,
    key_maxlen: u64,
}

impl<T: Read + Write> TcpTransport<T> {
    pub fn new(mut stream: T) -> TcpTransport<T> {
        TcpTransport {
            stream: stream,
            key_maxlen: 250, // memcached standard
        }
    }

    pub fn with_key_maxlen(&mut self,
                           key_maxlen: u64)
                           -> &mut TcpTransport<T> {
        self.key_maxlen = key_maxlen;
        self
    }


    pub fn read_byte(&mut self) -> TcpTransportResult<u8> {
        let mut bytes = [0; 1];

        match self.stream.read(&mut bytes) {
            Ok(1) => Ok(bytes[0]),
            _ => Err(TcpTransportError::SocketReadError),
        }
    }

    pub fn read_bytes(&mut self, len: u64) -> TcpTransportResult<Vec<u8>> {
        let mut bytes = vec![];

        for _ in 0..len {
            let byte = try!(self.read_byte());
            bytes.push(byte);
        }

        Ok(bytes)
    }

    pub fn read_line(&mut self, maxlen: usize) -> TcpTransportResult<Vec<u8>> {
        let mut bytes = vec![];
        let mut found_line_end = false;

        for _ in 0..maxlen {
            let byte = try!(self.read_byte());
            bytes.push(byte);

            // Look for \r\n
            if bytes.ends_with(&[13, 10]) {
                found_line_end = true;
                break;
            }
        }

        if found_line_end {
            // Pop off \r\n
            bytes.pop();
            bytes.pop();
            Ok(bytes)
        } else {
            Err(TcpTransportError::LineReadError)
        }
    }

    pub fn parse_word(&self,
                      bytes: Vec<u8>)
                      -> TcpTransportResult<(Vec<u8>, Vec<u8>)> {
        let mut space_idx = -1;

        for i in 0..bytes.len() {
            // We're looking for a space
            if bytes[i] == 32 {
                space_idx = i;
            }
        }

        if space_idx as i64 > -1 {
            let mut word = vec![];
            let mut rest = vec![];

            // TODO figure out how to return a modified vector instead of
            // copying the whole rest of it
            for i in 0..bytes.len() {
                let byte = bytes[i];
                if i < space_idx {
                    word.push(byte);
                } else if i == space_idx {
                    // we exclude the space from either slice
                } else {
                    rest.push(byte);
                }
            }

            Ok((word, rest))

        } else {
            // If we've reached the end of the buffer without seeing a space
            // that makes the whole buffer a word
            Ok((bytes, vec![]))
        }
    }


    pub fn read_cmd(&mut self) -> TcpTransportResult<Cmd> {
        // This needs to be the length of the longest command line, not
        // including data values for which the length is given upfront
        let line_len = self.key_maxlen as usize + 100;

        let fst_line = try!(self.read_line(line_len));
        let (fst_word, rest) = try!(self.parse_word(fst_line));
        let fst_word_str = String::from_utf8(fst_word).unwrap(); // XXX errors

        if fst_word_str == "stats" {
            return Ok(Cmd::Stats);
        }

        Err(TcpTransportError::InvalidCmd)
    }

    pub fn write_resp(&mut self, resp: &Resp) -> TcpTransportResult<()> {
        Ok(())
    }
}
