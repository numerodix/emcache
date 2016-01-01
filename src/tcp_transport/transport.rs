use std::cmp;
use std::io::Read;
use std::io::Write;

use bufstream::BufStream;

use protocol::cmd::Cmd;
use protocol::cmd::Get;
use protocol::cmd::Resp;
use protocol::cmd::Set;

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
            // much longer than the key itself. If it is ... XXX
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


    pub fn get_max_line_len(&self) -> usize {
        // This needs to be the length of the longest command line, not
        // including data values for which the length is given upfront
        self.key_maxlen as usize + 100
    }

    pub fn get_stats_clone(&self) -> TransportStats {
        self.stats.clone()
    }

    pub fn get_stream(&self) -> &T {
        self.stream.get_ref()
    }

    // Basic bytes manipulation and reading from the stream

    pub fn read_byte(&mut self) -> TcpTransportResult<u8> {
        let mut bytes = [0; 1];

        match self.stream.read(&mut bytes) {
            Ok(1) => {
                // Update stats
                self.stats.bytes_read += 1;

                Ok(bytes[0])
            }
            _ => Err(TcpTransportError::StreamReadError),
        }
    }

    pub fn read_bytes(&mut self, len: u64) -> TcpTransportResult<Vec<u8>> {
        let mut bytes = vec![0; len as usize];

        match self.stream.read(&mut bytes[..]) {
            Ok(n) => {
                // Update stats
                self.stats.bytes_read += n as u64;

                Ok(bytes)
            }
            _ => Err(TcpTransportError::StreamReadError),
        }
    }

    pub fn preread_line(&mut self) -> TcpTransportResult<()> {
        let mut cursor = 0;

        // We keep reading one byte at a time into line_buffer, looking for
        // a line terminator \r\n. The underlying stream is buffered.
        loop {
            let rv = self.stream.read(&mut self.line_buffer[cursor..cursor+1]);

            // If there was an error or if there was nothing to read we bail
            if rv.is_err() || rv.unwrap() == 0 {
                return Err(TcpTransportError::StreamReadError);
            }

            // We found \r
            if self.line_buffer[cursor] == 13 {
                // Read one more, hoping it's \n
                cursor += 1;
                let rv = self.stream.read(&mut self.line_buffer[cursor..cursor+1]);

                // If there was an error or if there was nothing to read we bail
                if rv.is_err() || rv.unwrap() == 0 {
                    return Err(TcpTransportError::StreamReadError);
                }

                // Woops, it's not \n, we bail
                if self.line_buffer[cursor] != 10 {
                    return Err(TcpTransportError::StreamReadError);
                }

                break;
            }

            cursor += 1;
        }

        self.line_cursor = 0;
        self.line_break_pos = cursor - 1;  // point it at \r, not \n

        Ok(())
    }

    pub fn line_is_empty(&self) -> bool {
        // If the cursor precedes the line break then there are still bytes to
        // read from the line, otherwise it's empty
        self.line_cursor >= self.line_break_pos
    }

    pub fn line_remove_first_char(&mut self) -> TcpTransportResult<()> {
        match !self.line_is_empty() {
            true => {
                self.line_cursor += 1;
                Ok(())
            }
            false => Err(TcpTransportError::LineReadError),
        }
    }

    pub fn line_parse_word(&mut self) -> TcpTransportResult<&[u8]> {
        // If the very first char is a space then our caller is out of sync
        if self.line_buffer[self.line_cursor] == 32 {
            return Err(TcpTransportError::StreamReadError);
        }

        let mut space_idx = 0;
        let mut found = false;

        for i in self.line_cursor + 1..self.line_break_pos {
            // We found a space
            if self.line_buffer[i] == 32 {
                space_idx = i;
                found = true;
                break;
            }
        }

        // If we didn't find a space then the whole line is a word
        // TODO test for this
        if !found {
            space_idx = self.line_break_pos;
        }

        // Advance the cursor, we've now "consumed" the word we found
        let word = &self.line_buffer[self.line_cursor..space_idx];
        self.line_cursor = space_idx;

        Ok(word)
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
        // remove the space after the keyword
        try!(self.line_remove_first_char());

        // parse the key
        let key_str = {
            let key = try!(self.line_parse_word());
            try!(as_string(key))
        };

        // We expect to find the end of the line now
        if self.line_is_empty() {
            Ok(Cmd::Get(Get { key: key_str }))
        } else {
            Err(TcpTransportError::CommandParseError)
        }
    }

    pub fn parse_cmd_set(&mut self) -> TcpTransportResult<Cmd> {
        // remove the space after the keyword
        try!(self.line_remove_first_char());

        // parse the key + remove trailing space
        let key_str = {
            let key = try!(self.line_parse_word());
            try!(as_string(key))
        };
        try!(self.line_remove_first_char());

        // parse the flags + remove trailing space
        let flags_num = {
            let flags = try!(self.line_parse_word());
            try!(as_number::<u16>(flags))
        };
        try!(self.line_remove_first_char());

        // parse the exptime + remove trailing space
        let exptime_num = {
            let exptime = try!(self.line_parse_word());
            try!(as_number::<u32>(exptime))
        };
        try!(self.line_remove_first_char());

        // parse the bytelen
        let bytelen_num = {
            let bytelen = try!(self.line_parse_word());
            try!(as_number::<u64>(bytelen))
        };

        // We know the byte length, so now read the value
        let value = try!(self.read_bytes(bytelen_num));

        // Read the line termination marker
        let newline = try!(self.read_bytes(2));  // TODO: verify newline

        // We got all the values we expected and there is nothing left
        return Ok(Cmd::Set(Set {
            key: key_str,
            exptime: exptime_num,
            data: value,
        }));

        //Err(TcpTransportError::CommandParseError)
    }

    // High level functions

    pub fn read_cmd(&mut self) -> TcpTransportResult<Cmd> {
        // read the first line
        try!(self.preread_line());

        let keyword_str = {
            let keyword = try!(self.line_parse_word());
            try!(as_string(keyword))
        };

        if keyword_str == "get" {
            return self.parse_cmd_get();
        } else if keyword_str == "set" {
            return self.parse_cmd_set();
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
