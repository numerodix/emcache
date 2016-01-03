use std::io::Read;
use std::io::Write;

use bufstream::BufStream;

use common::consts;
use protocol::cmd::Cmd;
use protocol::cmd::Get;
use protocol::cmd::Resp;
use protocol::cmd::Set;
use protocol::cmd::SetInstr;

use super::conversions::as_number;
use super::conversions::as_string;
use super::errors::TcpTransportError;
use super::stats::TransportStats;
use super::typedefs::TcpTransportResult;


pub struct TcpTransport<T: Read + Write> {
    stream: BufStream<T>,

    pub line_buffer: Vec<u8>,
    pub line_cursor: usize,
    pub line_break_pos: usize,

    key_maxlen: u64,

    stats: TransportStats,
}

impl<T: Read + Write> TcpTransport<T> {
    pub fn new(stream: T) -> TcpTransport<T> {
        TcpTransport {
            key_maxlen: 250, // memcached standard

            // Used to read the first line of a command, which includes a
            // keyword, a key, flags and a bytecount. We don't expect it to be
            // much longer than the key itself. If it is we panic...
            line_buffer: vec![0; 250 + 100],
            line_cursor: 0,
            line_break_pos: 0,

            stats: TransportStats::new(),
            stream: BufStream::new(stream),
        }
    }

    pub fn with_key_maxlen(&mut self,
                           key_maxlen: u64)
                           -> &mut TcpTransport<T> {
        self.key_maxlen = key_maxlen;
        self
    }


    pub fn get_stats_clone(&self) -> TransportStats {
        self.stats.clone()
    }

    pub fn get_stream(&self) -> &T {
        self.stream.get_ref()
    }

    // Basic bytes manipulation and reading from the stream

    pub fn read_bytes_exact(&mut self, len: u64) -> TcpTransportResult<Vec<u8>> {
        let mut bytes = vec![0; len as usize];
        let mut cursor: usize = 0;
        let mut iteration = 0;

        loop {
            // Read as much as we can, hopefully the whole buffer
            let rv = self.stream.read(&mut bytes[cursor..]);

            // Something went wrong
            if rv.is_err() {
                return Err(TcpTransportError::StreamReadError);
            }

            // How much we actually read
            let bytes_cnt = rv.unwrap();

            // Woops, there was nothing to read!
            if bytes_cnt == 0 {
                if iteration == 0 {
                    // It's the first iteration, so there wasn't anything to
                    // read in the first place, we were called in vain!
                    return Err(TcpTransportError::StreamReadError);

                } else {
                    // It turns out we read the very last byte on the last
                    // iteration, so nothing more to do at this point
                    break;
                }
            }

            // We advance the position in the buffer for next iteration
            cursor += bytes_cnt;

            // Update stats
            self.stats.bytes_read += bytes_cnt as u64;

            // We've read as much as was requested already
            if (bytes_cnt as u64) >= len {
                break;
            }

            iteration += 1;
        }

        if (cursor as u64) < len {
            bytes.truncate(cursor);
        }

        Ok(bytes)
    }

    pub fn read_word_in_line(&mut self) -> TcpTransportResult<(Vec<u8>, bool)> {
        let mut word = vec![];
        let mut byte = [0; 1];
        let mut end_of_line = false;

        loop {
            // Read a byte
            let rv = self.stream.read(&mut byte);

            // If there was an error or if there was nothing to read we bail
            if rv.is_err() || rv.unwrap() == 0 {
                return Err(TcpTransportError::StreamReadError);
            }

            // Update stats
            self.stats.bytes_read += 1;

            if byte[0] == consts::BYTE_SPACE {
                // We found a space

                if word.is_empty() {
                    // If it's one or more leading space we ignore it
                    continue;
                }

                // All good, we've found the end of the word
                break;

            } else if byte[0] == consts::BYTE_CARRIAGE_RETURN {
                // We found \r, we think it's the end of the line

                // Try to read \n
                let rv = self.stream.read(&mut byte);

                // If there was an error or if there was nothing to read we bail
                if rv.is_err() || rv.unwrap() == 0 {
                    return Err(TcpTransportError::StreamReadError);
                }

                // Update stats
                self.stats.bytes_read += 1;

                // If it's not a correct end of line we storm out in protest
                if byte[0] != consts::BYTE_LINE_FEED {
                    return Err(TcpTransportError::LineReadError);
                }

                // Else it's all good, we've read the whole line including the
                // terminator
                end_of_line = true;
                break;

            } else {
                // It's not a special char, append to our word
                word.push(byte[0]);
            }
        }

        Ok((word, end_of_line))
    }

    pub fn read_line_as_words(&mut self) -> TcpTransportResult<Vec<Vec<u8>>> {
        let mut words = vec![];

        loop {
            let (word, end_of_line) = try!(self.read_word_in_line());

            // Don't bother if it's an empty word (trailing space before \r\n)
            if !word.is_empty() {
                words.push(word);
            }

            if end_of_line {
                break;
            }
        }

        Ok(words)
    }

    // Writing to the stream

    pub fn flush_writes(&mut self) -> TcpTransportResult<()> {
        match self.stream.flush() {
            Ok(_) => Ok(()),
            Err(_) => Err(TcpTransportError::StreamWriteError),
        }
    }

    pub fn write_bytes(&mut self,
                       bytes: &Vec<u8>)
                       -> TcpTransportResult<usize> {
        match self.stream.write(bytes) {
            Ok(cnt_written) => {
                // Update stats
                self.stats.bytes_written += cnt_written as u64;

                Ok(cnt_written)
            }
            Err(_) => Err(TcpTransportError::StreamWriteError),
        }
    }

    pub fn write_string(&mut self, string: &str) -> TcpTransportResult<usize> {
        let bytes = string.to_string().into_bytes();
        Ok(try!(self.write_bytes(&bytes)))
    }


    // Parse individual commands

    pub fn parse_cmd_get(&mut self) -> TcpTransportResult<Cmd> {
        // parse the key
        let (key, end_of_line) = try!(self.read_word_in_line());
        let key_str = try!(as_string(key));

        if !end_of_line {
            return Err(TcpTransportError::CommandParseError);
        }

        Ok(Cmd::Get(Get { keys: vec![key_str] }))
    }

    pub fn parse_cmd_set(&mut self) -> TcpTransportResult<Cmd> {
        // parse the key
        let key_str = {
            let (key, end_of_line) = try!(self.read_word_in_line());

            if end_of_line {
                return Err(TcpTransportError::CommandParseError);
            }

            try!(as_string(key))
        };

        // parse the flags
        let flags_num = {
            let (flags, end_of_line) = try!(self.read_word_in_line());

            if end_of_line {
                return Err(TcpTransportError::CommandParseError);
            }

            try!(as_number::<u16>(flags))
        };

        // parse the exptime
        let exptime_num = {
            let (exptime, end_of_line) = try!(self.read_word_in_line());

            if end_of_line {
                return Err(TcpTransportError::CommandParseError);
            }

            try!(as_number::<u32>(exptime))
        };

        // parse the bytelen
        let bytelen_num = {
            let (bytelen, end_of_line) = try!(self.read_word_in_line());

            if end_of_line {
                return Err(TcpTransportError::CommandParseError);
            }

            try!(as_number::<u64>(bytelen))
        };

        // parse noreply
        let noreply_flag = {
            let (noreply, end_of_line) = try!(self.read_word_in_line());

            if !end_of_line {
                return Err(TcpTransportError::CommandParseError);
            }

            let noreply_str = try!(as_string(noreply));
            match noreply_str == "noreply" {
                true => true,
                false => false,
            }
        };

        // We now know the byte length, so read the value
        let value = try!(self.read_bytes_exact(bytelen_num));

        // The value is the wrong size
        if value.len() as u64 != bytelen_num {
            return Err(TcpTransportError::CommandParseError);
        }

        // Verify that we found the line terminator
        let terminator = try!(self.read_bytes_exact(2));
        if !terminator.ends_with(&[consts::BYTE_CARRIAGE_RETURN,
                                   consts::BYTE_LINE_FEED]) {
            return Err(TcpTransportError::CommandParseError);
        }

        // We got all the values we expected and there is nothing left
        return Ok(Cmd::Set(Set {
            instr: SetInstr::Set,
            key: key_str,
            flags: flags_num,
            exptime: exptime_num,
            data: value,
            noreply: noreply_flag,
        }));
    }

    // High level functions

    pub fn read_cmd(&mut self) -> TcpTransportResult<Cmd> {
        let keyword_str = {
            let (word, end_of_line) = try!(self.read_word_in_line());
            try!(as_string(word))
        };

        if keyword_str == "get" {
            // TODO check for !eol
            return self.parse_cmd_get();
        } else if keyword_str == "set" {
            // TODO check for !eol
            return self.parse_cmd_set();
        } else if keyword_str == "stats" {
            // TODO check for eol
            return Ok(Cmd::Stats);
        }

        Err(TcpTransportError::InvalidCmd)
    }

    pub fn write_resp(&mut self, resp: &Resp) -> TcpTransportResult<()> {
        match *resp {
            Resp::Empty => (),
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
                try!(self.write_string(" ")); // space
                try!(self.write_string(&value.flags.to_string())); // flags
                try!(self.write_string(" ")); // space
                try!(self.write_string(&value.data.len().to_string())); // bytelen
                try!(self.write_string(&"\r\n".to_string())); // newline
                try!(self.write_bytes(&value.data)); // data block
                try!(self.write_string(&"\r\n".to_string())); // newline
                try!(self.write_string(&"END\r\n".to_string())); // END + newline
            }
            _ => {
                return Err(TcpTransportError::StreamWriteError);
            }
        }

        // Make sure all bytes were actually sent
        self.flush_writes()
    }
}
