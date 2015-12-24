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
        let buflen = buf.len();
        let mut read_cnt = 0;

        for (i, byte) in self.incoming.iter().take(buflen).enumerate() {
            buf[i] = *byte;
            read_cnt += 1;
        }

        Ok(read_cnt)
    }
}

impl Write for TestStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut write_cnt = 0;

        for byte in buf.iter() {
            self.outgoing.push(*byte);
            write_cnt += 1;
        }

        Ok(write_cnt)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}


#[test]
fn test_stream_read() {
    let mut ts = TestStream::new(vec![1, 2, 3]);

    let mut buf = [0; 4];
    let read_cnt = ts.read(&mut buf).unwrap();

    assert_eq!(read_cnt, 3);
    assert_eq!(buf, [1, 2, 3, 0]);
}

#[test]
fn test_stream_write() {
    let mut ts = TestStream::new(vec![]);

    let buf = [1, 2, 3];
    let write_cnt = ts.write(&buf).unwrap();

    assert_eq!(write_cnt, 3);
    assert_eq!(ts.outgoing, [1, 2, 3]);
}
