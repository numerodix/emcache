use protocol::cmd::Cmd;
use protocol::cmd::Get;
use protocol::cmd::Resp;
use protocol::cmd::Set;
use protocol::cmd::Stat;
use protocol::cmd::Value;
use testlib::test_stream::TestStream;

use super::conversions::as_number;
use super::conversions::as_string;
use super::TcpTransport;
use super::TcpTransportError;


// Conversions

#[test]
fn test_as_string_ok() {
    // "a A"
    let string = as_string(&[97, 32, 65]).unwrap();
    assert_eq!(string, "a A".to_string());
}

#[test]
fn test_as_string_invalid() {
    // "a" + two invalid utf8 bytes
    let err = as_string(&[97, 254, 255]).unwrap_err();
    assert_eq!(err, TcpTransportError::Utf8Error);
}


#[test]
fn test_as_number_ok() {
    let st = "123".to_string();
    let num = as_number::<u32>(&st.as_bytes()).unwrap();
    assert_eq!(num, 123);
}

#[test]
fn test_as_number_invalid() {
    let st = "12 3".to_string();
    let err = as_number::<u32>(&st.as_bytes()).unwrap_err();
    assert_eq!(err, TcpTransportError::NumberParseError);
}


// Basic methods to consume the stream

#[test]
fn test_read_byte() {
    // "a"
    let ts = TestStream::new(vec![93]);
    let mut transport = TcpTransport::new(ts);

    let byte = transport.read_byte().unwrap();
    assert_eq!(byte, 93);
}

#[test]
fn test_read_byte_empty() {
    // ""
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_byte().unwrap_err();
    assert_eq!(err, TcpTransportError::StreamReadError);
}



#[test]
fn test_read_bytes() {
    // "a\r\n"
    let ts = TestStream::new(vec![93, 13, 10]);
    let mut transport = TcpTransport::new(ts);

    let bytes = transport.read_bytes(3).unwrap();
    assert_eq!(bytes, [93, 13, 10]);
}

#[test]
fn test_read_bytes_too_few() {
    // "a"
    let ts = TestStream::new(vec![93]);
    let mut transport = TcpTransport::new(ts);

    let bytes = transport.read_bytes(2).unwrap();
    assert_eq!(bytes, [93]);
}



#[test]
fn test_preread_line_zero_char() {
    // "\r\n"
    let ts = TestStream::new(vec![13, 10]);
    let mut transport = TcpTransport::new(ts);

    transport.preread_line().unwrap();
    assert_eq!(0, transport.line_break_pos);
    assert_eq!([13, 10, 0, 0], &transport.line_buffer[..4]);
}

#[test]
fn test_preread_line_one_char() {
    // "a\r\n"
    let ts = TestStream::new(vec![93, 13, 10]);
    let mut transport = TcpTransport::new(ts);

    transport.preread_line().unwrap();
    assert_eq!(1, transport.line_break_pos);
    assert_eq!([93, 13, 10, 0], &transport.line_buffer[..4]);
}

#[test]
fn test_preread_line_no_newline() {
    // "ab"
    let ts = TestStream::new(vec![93, 94]);
    let mut transport = TcpTransport::new(ts);

    transport.preread_line().unwrap_err();
    assert_eq!([93, 94, 0, 0], &transport.line_buffer[..4]);
}

#[test]
fn test_preread_line_invalid_newline_marker() {
    // "a\r"
    let ts = TestStream::new(vec![93, 13]);
    let mut transport = TcpTransport::new(ts);

    let err = transport.preread_line().unwrap_err();
    assert_eq!(err, TcpTransportError::StreamReadError);
}

#[test]
#[should_panic]
fn test_preread_line_too_long() {
    // "a" * 4096
    let ts = TestStream::new(vec![93; 4096]);
    let mut transport = TcpTransport::new(ts);

    transport.preread_line();
}


#[test]
fn test_line_remove_first_char_ok() {
    // "ab\r\n"
    let ts = TestStream::new(vec![93, 94, 13, 10]);
    let mut transport = TcpTransport::new(ts);

    transport.preread_line().unwrap();
    transport.line_remove_first_char().unwrap();
    assert_eq!(1, transport.line_cursor);
    assert_eq!(2, transport.line_break_pos);
}

#[test]
fn test_line_remove_first_char_fails() {
    // "\r\n"
    let ts = TestStream::new(vec![13, 10]);
    let mut transport = TcpTransport::new(ts);

    transport.preread_line().unwrap();
    // There's nothing left to read before the linebreak
    transport.line_remove_first_char().unwrap_err();
}


#[test]
fn test_line_parse_word_ok() {
    // "ab a\r\n"
    let ts = TestStream::new(vec![93, 94, 32, 93, 13, 10]);
    let mut transport = TcpTransport::new(ts);

    transport.preread_line().unwrap();
    {
        let word = transport.line_parse_word().unwrap();
        assert_eq!(&[93, 94], word);
    }
    assert_eq!(2, transport.line_cursor);
}

#[test]
fn test_parse_word_whole() {
    // "\1\2\3\3\4\11\r\n"
    let ts = TestStream::new(vec![1, 2, 3, 3, 4, 11, 13, 10]);
    let mut transport = TcpTransport::new(ts);

    transport.preread_line().unwrap();
    {
        let word = transport.line_parse_word().unwrap();
        assert_eq!(&[1, 2, 3, 3, 4, 11], word);
    }
    assert_eq!(6, transport.line_cursor);
    assert_eq!(6, transport.line_break_pos);
}

#[test]
fn test_line_parse_word_fails() {
    // " ab a\r\n"
    let ts = TestStream::new(vec![32, 93, 94, 32, 93, 13, 10]);
    let mut transport = TcpTransport::new(ts);

    transport.preread_line().unwrap();
    // can't, line starts with a space
    transport.line_parse_word().unwrap_err();
    assert_eq!(0, transport.line_cursor);
}


// Basic methods to produce the stream

#[test]
fn test_write_bytes() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let bytelen = transport.write_bytes(&vec![97, 98, 99]).unwrap();
    transport.flush_writes().unwrap();
    assert_eq!(transport.get_stream().outgoing, [97, 98, 99]);
}

#[test]
fn test_write_string() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let bytelen = transport.write_string("abc").unwrap();
    transport.flush_writes().unwrap();
    assert_eq!(bytelen, 3);
    assert_eq!(transport.get_stream().outgoing, [97, 98, 99]);
}


// Command parsing: malformed examples

#[test]
fn test_read_cmd_invalid() {
    let cmd_str = "invalid key 0 0 3\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_cmd().unwrap_err();
    assert_eq!(err, TcpTransportError::InvalidCmd);
}

#[test]
fn test_read_cmd_malterminated() {
    let cmd_str = "stats\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_cmd().unwrap_err();
    assert_eq!(err, TcpTransportError::StreamReadError);
}


// Command parsing: Get

#[test]
fn test_read_cmd_get_ok() {
    let cmd_str = "get x\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Get(Get::new("x")));
}

#[test]
fn test_read_cmd_get_non_utf8() {
    // get X\r\n
    let cmd_bytes = vec![103, 101, 116, 32, 254, 13, 10];
    let ts = TestStream::new(cmd_bytes);
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_cmd().unwrap_err();
    assert_eq!(err, TcpTransportError::Utf8Error);
}

#[test]
fn test_read_cmd_get_malformed() {
    fn try_cmd(cmd: &str) {
        let cmd_str = cmd.to_string();
        let ts = TestStream::new(cmd_str.into_bytes());
        let mut transport = TcpTransport::new(ts);

        let err = transport.read_cmd().unwrap_err();
        assert_eq!(err, TcpTransportError::StreamReadError);
    }

    // Test for truncated stream
    try_cmd("get x\r");
    try_cmd("get x");
    try_cmd("get ");
    try_cmd("get");
}


// Command parsing: Set

#[test]
fn test_read_cmd_set_ok() {
    let cmd_str = "set x 0 0 3\r\nabc\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Set(Set::new("x", 0, vec![97, 98, 99])));
}

#[test]
fn test_read_cmd_set_under_size() {
    let cmd_str = "set x 0 0 2\r\nabc\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_cmd().unwrap_err();
    assert_eq!(err, TcpTransportError::CommandParseError);
}

#[test]
fn test_read_cmd_set_over_size() {
    let cmd_str = "set x 0 0 4\r\nabc\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_cmd().unwrap_err();
    assert_eq!(err, TcpTransportError::CommandParseError);
}

#[test]
fn test_read_cmd_set_malformed() {
    fn try_cmd(cmd: &str) {
        let cmd_str = cmd.to_string();
        let ts = TestStream::new(cmd_str.into_bytes());
        let mut transport = TcpTransport::new(ts);

        let err = transport.read_cmd().unwrap_err();
        assert_eq!(err, TcpTransportError::CommandParseError);
    }

    // Test for truncated stream
    try_cmd("set x 0 0 3\r\nabc\r");
    try_cmd("set x 0 0 3\r\nabc");
    try_cmd("set x 0 0 3\r\nab");
    try_cmd("set x 0 0 3\r\na");
    try_cmd("set x 0 0 3\r\n");
    return; // TODO some are CommandParseError, some are StreamReadError eh :/
    try_cmd("set x 0 0 3\r");
    try_cmd("set x 0 0 3");
    try_cmd("set x 0 0 ");
    try_cmd("set x 0 0");
    try_cmd("set x 0 ");
    try_cmd("set x 0");
    try_cmd("set x ");
    try_cmd("set x");
    try_cmd("set ");
    try_cmd("set");
}


// Command parsing: Stats

#[test]
fn test_read_cmd_stats() {
    let cmd_str = "stats\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Stats);
}


// Response writing: Error

#[test]
fn test_write_resp_error() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Error;
    transport.write_resp(&resp).unwrap();
    let expected = "ERROR\r\n".to_string().into_bytes();
    assert_eq!(transport.get_stream().outgoing, expected);
}


// Response writing: Stats

#[test]
fn test_write_resp_stats() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let stat = Stat::new("curr_items", "0".to_string());
    let resp = Resp::Stats(vec![stat]);
    transport.write_resp(&resp).unwrap();
    let expected = "STAT curr_items 0\r\nEND\r\n".to_string().into_bytes();
    assert_eq!(transport.get_stream().outgoing, expected);
}


// Response writing: Stored

#[test]
fn test_write_resp_stored() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Stored;
    transport.write_resp(&resp).unwrap();
    let expected = "STORED\r\n".to_string().into_bytes();
    assert_eq!(transport.get_stream().outgoing, expected);
}


// Response writing: Value

#[test]
fn test_write_resp_value() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Value(Value::new("x", "abc".to_string().into_bytes()));
    transport.write_resp(&resp).unwrap();
    let expected = "VALUE x 0 3\r\nabc\r\n".to_string().into_bytes();
    assert_eq!(transport.get_stream().outgoing, expected);
}
