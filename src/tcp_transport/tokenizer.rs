use std::io::Read;
use std::io::Write;
use std::mem;
use std::str::FromStr;

use super::conversions::as_number;
use super::conversions::as_string;
use super::errors::TcpTransportError;
use super::typedefs::TcpTransportResult;


#[derive(Debug, PartialEq)]
pub enum Token {
    Bytes(Vec<u8>),
    LineTerminator(Vec<u8>),
    Space(u8),
    Word(Vec<u8>),
}


pub struct Tokenizer<T: Read + Write> {
    stream: T,
    token_buffer: Option<Token>,

    bytes_read: u64,
}

impl<T: Read + Write> Tokenizer<T> {
    pub fn new(stream: T) -> Tokenizer<T> {
        Tokenizer { 
            stream: stream,
            token_buffer: None,
            bytes_read: 0,
        }
    }


    pub fn read_bytes_exact(&mut self,
                            len: u64)
                            -> TcpTransportResult<Token> {
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
            self.bytes_read += bytes_cnt as u64;

            // We've read as much as was requested already
            if (bytes_cnt as u64) >= len {
                break;
            }

            iteration += 1;
        }

        if (cursor as u64) < len {
            bytes.truncate(cursor);
        }

        Ok(Token::Bytes(bytes))
    }

    pub fn read_token(&mut self) -> TcpTransportResult<Token> {
        let mut word = vec![];
        let mut byte = [0; 1];

        loop {
            // Read a byte
            let rv = self.stream.read(&mut byte);

            // If there was an error or if there was nothing to read we bail
            if rv.is_err() || rv.unwrap() == 0 {
                return Err(TcpTransportError::StreamReadError);
            }

            // Update stats
            self.bytes_read += 1;

            if byte[0] == b' ' {
                // We found a space
                return Ok(Token::Space(byte[0]));

            } else if byte[0] == b'\r' {
                // We found \r, we think it's the end of the line

                // Try to read \n
                let rv = self.stream.read(&mut byte);

                // If there was an error or if there was nothing to read we bail
                if rv.is_err() || rv.unwrap() == 0 {
                    return Err(TcpTransportError::StreamReadError);
                }

                // Update stats
                self.bytes_read += 1;

                // If it's not a correct end of line we storm out in protest
                if byte[0] != b'\n' {
                    return Err(TcpTransportError::LineReadError);
                }

                // Else it's all good, we've read the whole line including the
                // terminator
                return Ok(Token::LineTerminator(vec![b'\r', b'\n']));

            } else {
                // It's not a special char, append to our word
                word.push(byte[0]);
            }
        }

        Ok(Token::Word(word))
    }



    pub fn maybe_fill_token_buffer(&mut self) -> TcpTransportResult<()> {
        if self.token_buffer.is_none() {
            self.token_buffer = Some(try!(self.read_token()));
        }

        Ok(())
    }

    pub fn flush_token_buffer(&mut self) -> TcpTransportResult<()> {
        self.token_buffer = None;

        Ok(())
    }

    pub fn get_next_token(&mut self) -> TcpTransportResult<Token> {
        try!(self.maybe_fill_token_buffer());

        // Release the token in the buffer so we can return it
        let token = mem::replace(&mut self.token_buffer, None);

        Ok(token.unwrap())
    }

    pub fn peek_next_token(&mut self) -> TcpTransportResult<&Token> {
        try!(self.maybe_fill_token_buffer());

        match self.token_buffer {
            Some(ref token) => Ok(token),
            _ => Err(TcpTransportError::StreamReadError),
        }
    }



    pub fn read_word_as_number<N: FromStr>(&mut self) -> TcpTransportResult<N> {
        let rv = {
            let token = try!(self.peek_next_token());
            match token {
                &Token::Word(ref word) => {
                    as_number(word.clone())
                }
                _ => Err(TcpTransportError::WrongToken)
            }
        };

        match rv {
            Ok(num) => {
                // We have a successful value to return, so now empty the buffer
                try!(self.flush_token_buffer());

                Ok(num)
            }
            Err(e) => Err(e),
        }
    }

    pub fn read_word_as_string(&mut self) -> TcpTransportResult<String> {
        let rv = {
            let token = try!(self.peek_next_token());
            match token {
                &Token::Word(ref word) => {
                    as_string(word.clone())
                }
                _ => Err(TcpTransportError::WrongToken)
            }
        };

        match rv {
            Ok(st) => {
                // We have a successful value to return, so now empty the buffer
                try!(self.flush_token_buffer());

                Ok(st)
            }
            Err(e) => Err(e),
        }
    }

    pub fn next_word_is_string(&mut self, value: &str) -> TcpTransportResult<bool> {
        let rv = {
            let token = try!(self.peek_next_token());
            match token {
                &Token::Word(ref word) => {
                    as_string(word.clone())
                }
                _ => Err(TcpTransportError::WrongToken)
            }
        };

        match rv {
            Ok(st) => {
                if value != st {
                    return Err(TcpTransportError::CommandParseError);
                }

                // We have a successful value to return, so now empty the buffer
                try!(self.flush_token_buffer());

                Ok(true)
            }
            Err(e) => Err(e),
        }
    }


    pub fn read_line_terminator(&mut self) -> TcpTransportResult<Token> {
        let token = try!(self.read_bytes_exact(2));

        match token {
            Token::Bytes(bytes) => match &bytes[..] {
                b"\r\n" => {
                    Ok(Token::LineTerminator(vec![b'\r', b'\n']))
                }
                _ => Err(TcpTransportError::StreamReadError),
            },
            _ => Err(TcpTransportError::WrongToken),
        }
    }
}
