use std::io::Read;
use std::io::Write;
use std::str::FromStr;

use bufstream::BufStream;

use protocol::cmd::Cmd;
use protocol::cmd::Get;
use protocol::cmd::Resp;
use protocol::cmd::Set;

use super::errors::TcpTransportError;
use super::stats::TransportStats;
use super::typedefs::TcpTransportResult;


pub struct TcpTransport<T: Read + Write> {
    stream: BufStream<T>,
    // queue up response data before writing to the stream
    outgoing_buffer: Vec<u8>,

    metrics: TransportStats,
    key_maxlen: u64,
}

impl<T: Read + Write> TcpTransport<T> {
    pub fn new(stream: T) -> TcpTransport<T> {
        TcpTransport {
            key_maxlen: 250, // memcached standard
            metrics: TransportStats::new(),
            outgoing_buffer: vec![],
            stream: BufStream::new(stream),
        }
    }

    pub fn with_key_maxlen(&mut self,
                           key_maxlen: u64)
                           -> &mut TcpTransport<T> {
        self.key_maxlen = key_maxlen;
        self
    }


    pub fn get_max_line_len(&self) -> usize {
        // This needs to be the length of the longest command line, not
        // including data values for which the length is given upfront
        self.key_maxlen as usize + 100
    }

    pub fn get_stats_clone(&self) -> TransportStats {
        self.metrics.clone()
    }

    pub fn get_stream(&self) -> &T {
        self.stream.get_ref()
    }

    pub fn get_outgoing_buffer(&self) -> &Vec<u8> {
        &self.outgoing_buffer
    }

    // Basic bytes manipulation and reading from the stream

    pub fn as_string(&self, bytes: Vec<u8>) -> TcpTransportResult<String> {
        match String::from_utf8(bytes) {
            Ok(string) => Ok(string),
            Err(_) => Err(TcpTransportError::Utf8Error),
        }
    }

    pub fn as_number<N: FromStr>(&self,
                                 bytes: Vec<u8>)
                                 -> TcpTransportResult<N> {
        let string = try!(self.as_string(bytes));
        match string.parse::<N>() {
            Ok(num) => Ok(num),
            Err(_) => Err(TcpTransportError::NumberParseError),
        }
    }

    pub fn read_byte(&mut self) -> TcpTransportResult<u8> {
        let mut bytes = [0; 1];

        match self.stream.read(&mut bytes) {
            Ok(1) => {
                // Update metrics
                self.metrics.bytes_read += 1;

                Ok(bytes[0])
            }
            _ => Err(TcpTransportError::StreamReadError),
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

    pub fn remove_first_char(&self,
                             bytes: &mut Vec<u8>)
                             -> TcpTransportResult<()> {
        match bytes.len() > 0 {
            true => {
                bytes.remove(0);
                Ok(())
            }
            false => Err(TcpTransportError::StreamReadError),
        }
    }

    pub fn parse_word(&self,
                      bytes: Vec<u8>)
                      -> TcpTransportResult<(Vec<u8>, Vec<u8>)> {
        let mut space_idx: i64 = -1;

        for i in 0..bytes.len() {
            // We're looking for a space
            if bytes[i] == 32 {
                space_idx = i as i64;
                break;
            }
        }

        if space_idx > -1 {
            let mut word = vec![];
            let mut rest = vec![];

            // TODO figure out how to return a modified vector instead of
            // copying the whole rest of it
            for i in 0..bytes.len() {
                let byte = bytes[i];
                if (i as i64) < space_idx {
                    word.push(byte);
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

    // Writing to the stream

    pub fn write_bytes(&mut self,
                       bytes: &Vec<u8>)
                       -> TcpTransportResult<usize> {
        for i in 0..bytes.len() {
            let byte = bytes[i];
            self.outgoing_buffer.push(byte);
        }

        Ok(bytes.len())
    }

    pub fn write_string(&mut self, string: &str) -> TcpTransportResult<usize> {
        let bytes = string.to_string().into_bytes();
        Ok(try!(self.write_bytes(&bytes)))
    }

    pub fn flush_writes(&mut self) -> TcpTransportResult<()> {
        let rv = self.stream.write(&self.outgoing_buffer);
        self.outgoing_buffer.clear();

        if !rv.is_ok() {
            return Err(TcpTransportError::StreamWriteError);
        }

        let cnt_written = rv.unwrap();

        // Update metrics
        self.metrics.bytes_written += cnt_written as u64;

        let rv = self.stream.flush();

        match rv {
            Ok(_) => Ok(()),
            Err(_) => Err(TcpTransportError::StreamWriteError),
        }
    }

    // Parse individual commands

    pub fn parse_cmd_get(&self, mut rest: Vec<u8>) -> TcpTransportResult<Cmd> {
        try!(self.remove_first_char(&mut rest)); // remove leading space
        let (key, rest) = try!(self.parse_word(rest));

        // We expect to find the end of the line now
        if rest.is_empty() {
            let key_str = try!(self.as_string(key));
            Ok(Cmd::Get(Get { key: key_str }))
        } else {
            Err(TcpTransportError::CommandParseError)
        }
    }

    pub fn parse_cmd_set(&mut self,
                         mut rest: Vec<u8>)
                         -> TcpTransportResult<Cmd> {
        try!(self.remove_first_char(&mut rest)); // remove leading space
        let (key, mut rest) = try!(self.parse_word(rest));

        try!(self.remove_first_char(&mut rest)); // remove leading space
        let (flags, mut rest) = try!(self.parse_word(rest));

        try!(self.remove_first_char(&mut rest)); // remove leading space
        let (exptime, mut rest) = try!(self.parse_word(rest));

        try!(self.remove_first_char(&mut rest)); // remove leading space
        let (bytelen, _) = try!(self.parse_word(rest));

        let key_str = try!(self.as_string(key));
        let flags_num = try!(self.as_number::<u16>(flags));
        let exptime_num = try!(self.as_number::<u32>(exptime));
        let bytelen_num = try!(self.as_number::<u64>(bytelen));

        // We know the byte length, so now read the value
        let value = try!(self.read_bytes(bytelen_num));

        // Read the line termination marker
        let line_len = self.get_max_line_len();
        let rest = try!(self.read_line(line_len));

        // We got all the values we expected and there is nothing left
        if rest.is_empty() {
            return Ok(Cmd::Set(Set {
                key: key_str,
                exptime: exptime_num,
                data: value,
            }));
        }

        Err(TcpTransportError::CommandParseError)
    }

    // High level functions

    pub fn read_cmd(&mut self) -> TcpTransportResult<Cmd> {
        let line_len = self.get_max_line_len();

        let fst_line = try!(self.read_line(line_len));
        let (keyword, rest) = try!(self.parse_word(fst_line));
        let keyword_str = try!(self.as_string(keyword));

        if keyword_str == "get" {
            return self.parse_cmd_get(rest);
        } else if keyword_str == "set" {
            return self.parse_cmd_set(rest);
        } else if keyword_str == "stats" {
            return Ok(Cmd::Stats);
        }

        Err(TcpTransportError::InvalidCmd)
    }

    pub fn write_resp(&mut self, resp: &Resp) -> TcpTransportResult<()> {
        match *resp {
            Resp::Error => {
                try!(self.write_string("ERROR\r\n"));
            }
            Resp::Stats(ref stats) => {
                for stat in stats {
                    try!(self.write_string("STAT "));
                    try!(self.write_string(&stat.key));
                    try!(self.write_string(" "));
                    try!(self.write_string(&stat.value));
                    try!(self.write_string("\r\n"));
                }
                try!(self.write_string("END\r\n"));
            }
            Resp::Stored => {
                try!(self.write_string("STORED\r\n"));
            }
            Resp::Value(ref value) => {
                try!(self.write_string("VALUE ")); // keyword
                try!(self.write_string(&value.key)); // key
                try!(self.write_string(" 0 ")); // flags
                try!(self.write_string(&value.data.len().to_string())); // bytelen
                try!(self.write_string(&"\r\n".to_string())); // newline
                try!(self.write_bytes(&value.data)); // data block
                try!(self.write_string(&"\r\n".to_string())); // newline
            }
            _ => {
                return Err(TcpTransportError::StreamWriteError);
            }
        }

        // Make sure all bytes were actually sent
        self.flush_writes()
    }
}
