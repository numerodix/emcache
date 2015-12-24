use super::TcpTransport;
use super::test_stream::TestStream;


#[test]
fn test_x() {
    let mut ts = TestStream::new(vec![93, 13, 10]);
    let mut transport = TcpTransport::new(ts);

    let rv = transport.read_byte().unwrap();
    println!("read: {:?}", rv);
}
