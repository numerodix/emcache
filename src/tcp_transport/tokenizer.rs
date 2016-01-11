use std::io::Read;
use std::io::Write;

use super::errors::TcpTransportError;
use super::typedefs::TcpTransportResult;


enum ReadingMode {
    LineMode,
    BytesMode,
}


pub struct Tokenizer<T: Read + Write> {
    stream: T,

    mode: ReadingMode,

    bytes_read: u64,
}

impl<T: Read + Write> Tokenizer<T> {
    pub fn new(stream: T) -> Tokenizer<T> {
        Tokenizer { 
            stream: stream,
            mode: ReadingMode::BytesMode,
            bytes_read: 0,
        }
    }

    pub fn read_bytes_exact(&mut self,
                            len: u64)
                            -> TcpTransportResult<Vec<u8>> {
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

        Ok(bytes)
    }

    pub fn read_word(&mut self) -> TcpTransportResult<(Vec<u8>, bool)> {
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
            self.bytes_read += 1;

            if byte[0] == b' ' {
                // We found a space

                if word.is_empty() {
                    // If it's one or more leading space we ignore it
                    continue;
                }

                // All good, we've found the end of the word
                break;

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
                end_of_line = true;
                break;

            } else {
                // It's not a special char, append to our word
                word.push(byte[0]);
            }
        }

        Ok((word, end_of_line))
    }

}
