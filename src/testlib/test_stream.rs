use std::cmp;
use std::io::Read;
use std::io::Write;
use std::io::Result;


// A stream that is seeded with incoming data which can be consumed and
// records data written to it (like a socket).
//
// This allows us to unit test a transport without using sockets. :)
pub struct TestStream {
    pub incoming: Vec<u8>,
    pub incoming_cursor: usize,

    pub outgoing: Vec<u8>,
}

impl TestStream {
    pub fn new(incoming: Vec<u8>) -> TestStream {
        TestStream {
            incoming: incoming,
            incoming_cursor: 0,

            // Should be a good fit for most test responses
            outgoing: Vec::with_capacity(200),
        }
    }
}

impl Read for TestStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // We can only read either as much as we have in incoming, or as big as
        // the output buffer is.
        let read_len = cmp::min(buf.len(),
                                self.incoming.len() - self.incoming_cursor);

        for i in 0..read_len {
            buf[i] = self.incoming[self.incoming_cursor + i];
        }
        self.incoming_cursor += read_len;

        Ok(read_len)
    }
}

impl Write for TestStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.outgoing.extend(buf.iter().cloned());

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}


#[test]
fn test_stream_read_whole() {
    let mut ts = TestStream::new(vec![1, 2, 3]);

    let mut buf = [0; 4];
    let read_cnt = ts.read(&mut buf).unwrap();
    assert_eq!(buf, [1, 2, 3, 0]);
    assert_eq!(read_cnt, 3);
    assert_eq!(ts.incoming, [1, 2, 3]);
    assert_eq!(ts.incoming_cursor, 3);
}

#[test]
fn test_stream_read_incremental() {
    let mut ts = TestStream::new(vec![1, 2, 3]);

    // Read once
    let mut buf = [0; 2];
    let read_cnt = ts.read(&mut buf).unwrap();
    assert_eq!(read_cnt, 2);
    assert_eq!(buf, [1, 2]);
    assert_eq!(ts.incoming, [1, 2, 3]);
    assert_eq!(ts.incoming_cursor, 2);

    // Read once more
    let mut buf = [0; 2];
    let read_cnt = ts.read(&mut buf).unwrap();
    assert_eq!(read_cnt, 1);
    assert_eq!(buf, [3, 0]);
    assert_eq!(ts.incoming, [1, 2, 3]);
    assert_eq!(ts.incoming_cursor, 3);
}

#[test]
fn test_stream_write() {
    let mut ts = TestStream::new(vec![]);

    // Write once
    let buf = [1, 2];
    let write_cnt = ts.write(&buf).unwrap();
    assert_eq!(write_cnt, 2);
    assert_eq!(ts.outgoing, [1, 2]);

    // Write once more
    let buf = [3, 4, 5];
    let write_cnt = ts.write(&buf).unwrap();
    assert_eq!(write_cnt, 3);
    assert_eq!(ts.outgoing, [1, 2, 3, 4, 5]);
}
