use std::cmp;
use std::io::Read;
use std::io::Write;
use std::io::Result;

use super::TcpTransport;


// A stream that is seeded with incoming data which can be consumed and
// records data written to it (like a socket)
pub struct TestStream {
    pub incoming: Vec<u8>,
    pub outgoing: Vec<u8>,
}

impl TestStream {
    pub fn new(incoming: Vec<u8>) -> TestStream {
        TestStream {
            incoming: incoming,
            outgoing: vec![],
        }
    }
}

impl Read for TestStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let read_len = cmp::min(buf.len(), self.incoming.len());

        for i in 0..read_len {
            let byte = self.incoming.remove(0);
            buf[i] = byte;
        }

        Ok(read_len)
    }
}

impl Write for TestStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        for byte in buf.iter() {
            self.outgoing.push(*byte);
        }

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
    assert_eq!(ts.incoming, []);
}

#[test]
fn test_stream_read_incremental() {
    let mut ts = TestStream::new(vec![1, 2, 3]);

    // Read once
    let mut buf = [0; 2];
    let read_cnt = ts.read(&mut buf).unwrap();
    assert_eq!(read_cnt, 2);
    assert_eq!(buf, [1, 2]);
    assert_eq!(ts.incoming, [3]);

    // Read once more
    let mut buf = [0; 2];
    let read_cnt = ts.read(&mut buf).unwrap();
    assert_eq!(read_cnt, 1);
    assert_eq!(buf, [3, 0]);
    assert_eq!(ts.incoming, []);
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
