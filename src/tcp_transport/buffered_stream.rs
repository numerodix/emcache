use std::cmp;
use std::io::Read;
use std::io::Write;
use std::io::Result;


// A buffered stream that reads blocks from the underlying stream (possibly
// socket) into an internal buffer. The size of the input buffer is given
// upfront.
//
// For writing an output buffer is used, which will grow to any size and the
// underlying stream is not written into until flush() is called.

pub struct BufferedStream<T> {
    stream: T,

    incoming_cursor: usize,
    incoming_buffer: Vec<u8>,

    outgoing_buffer: Vec<u8>,
}


impl<T: Read + Write> BufferedStream<T> {
    pub fn new(stream: T,
               incoming_size: usize,
               outgoing_size_hint: usize)
               -> BufferedStream<T> {
        BufferedStream {
            incoming_buffer: Vec::with_capacity(incoming_size),
            outgoing_buffer: Vec::with_capacity(outgoing_size_hint),
            incoming_cursor: 0,
            stream: stream,
        }
    }
}


impl<T: Read> Read for BufferedStream<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut cum_read_size = 0;

        loop {
            // If our buffer is empty we need to fill it before we can satisfy
            // reads
            if self.incoming_buffer.is_empty() {
                try!(self.stream.read(&mut self.incoming_buffer));
                self.incoming_cursor = 0;
            }

            // What we're going to read in this round is either the full array
            // size or what is left in our buffer. In the latter case, if we
            // need more, we have to loop around.
            let read_size = cmp::min(buf.len(),
                                     self.incoming_buffer.capacity() -
                                     self.incoming_cursor + 1);

            // Copy bytes from our buffer at the cursor into the array
            for i in 0..read_size {
                buf[i] = self.incoming_buffer[i + self.incoming_cursor];
            }

            // Update the cursor
            self.incoming_cursor += read_size;
            // Update the overall read size
            cum_read_size += read_size;

            // If we've already read as much as the array can fit that means
            // we can't read any more.
            if cum_read_size >= buf.len() {
                break;
            }
        }

        Ok(cum_read_size)
    }
}


impl<T: Write> Write for BufferedStream<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // We can just write these bytes into our buffer; if it's not large
        // enough it will grow
        for b in buf {
            self.outgoing_buffer.push(*b);
        }

        // We guarantee we can write the full array size
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        // We write our whole buffer into the underlying stream then empty our
        // buffer
        try!(self.stream.write(&self.outgoing_buffer));
        self.outgoing_buffer.clear();

        Ok(())
    }
}
