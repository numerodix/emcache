use super::TcpTransport;
use super::TcpTransportError;
use super::test_stream::TestStream;

use protocol::cmd::Cmd;


#[test]
fn test_read_byte() {
    let mut ts = TestStream::new(vec![93]);
    let mut transport = TcpTransport::new(ts);

    let byte = transport.read_byte().unwrap();
    assert_eq!(byte, 93);
}

#[test]
fn test_read_bytes() {
    let mut ts = TestStream::new(vec![93, 13, 10]);
    let mut transport = TcpTransport::new(ts);

    let bytes = transport.read_bytes(3).unwrap();
    assert_eq!(bytes, [93, 13, 10]);
}

#[test]
fn test_read_line_ok() {
    let mut ts = TestStream::new(vec![93, 13, 10]);
    let mut transport = TcpTransport::new(ts);

    let line = transport.read_line(3).unwrap();
    assert_eq!(line, [93]);
}

#[test]
fn test_read_line_invalid_newline_marker() {
    let mut ts = TestStream::new(vec![93, 10]);
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_line(2).unwrap_err();
    assert_eq!(err, TcpTransportError::LineReadError);
}

#[test]
fn test_read_line_too_long() {
    let mut ts = TestStream::new(vec![93, 1, 2, 3, 13, 10]);
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_line(5).unwrap_err();
    assert_eq!(err, TcpTransportError::LineReadError);
}

#[test]
fn test_parse_word_ok() {
    let mut ts = TestStream::new(vec![1, 2, 32, 3, 4, 11]);
    let mut transport = TcpTransport::new(ts);

    let bytes = transport.read_bytes(6).unwrap();
    let word = transport.parse_word(bytes).unwrap();
    assert_eq!(word, [1, 2]);
}

#[test]
fn test_parse_word_failed() {
    let mut ts = TestStream::new(vec![1, 2, 3, 3, 4, 11]);
    let mut transport = TcpTransport::new(ts);

    let bytes = transport.read_bytes(6).unwrap();
    let word = transport.parse_word(bytes).unwrap();
    assert_eq!(word, [1, 2, 3, 3, 4, 11]);
}


#[test]
fn test_read_cmd_invalid() {
    let cmd_str = "invalid key 0 0 3\r\n".to_string();
    let mut ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_cmd().unwrap_err();
    assert_eq!(err, TcpTransportError::InvalidCmd);
}

#[test]
fn test_read_cmd_malterminated() {
    let cmd_str = "stats\n".to_string();
    let mut ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_cmd().unwrap_err();
    assert_eq!(err, TcpTransportError::SocketReadError);
}

#[test]
fn test_read_cmd_stats() {
    let cmd_str = "stats\r\n".to_string();
    let mut ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Stats);
}
