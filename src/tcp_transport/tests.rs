use protocol::cmd::Cmd;
use protocol::cmd::Delete;
use protocol::cmd::Get;
use protocol::cmd::Resp;
use protocol::cmd::Set;
use protocol::cmd::SetInstr;
use protocol::cmd::Stat;
use protocol::cmd::Value;
use testlib::test_stream::TestStream;

use super::TcpTransport;
use super::TcpTransportError;
use super::conversions::as_number;
use super::conversions::as_string;


// Conversions

#[test]
fn test_as_string_ok() {
    // "a A"
    let string = as_string(vec![97, 32, 65]).unwrap();
    assert_eq!(string, "a A".to_string());
}

#[test]
fn test_as_string_invalid() {
    // "a" + two invalid utf8 bytes
    let err = as_string(vec![97, 254, 255]).unwrap_err();
    assert_eq!(err, TcpTransportError::Utf8Error);
}


#[test]
fn test_as_number_ok() {
    let bytes = "123".to_string().into_bytes();
    let num = as_number::<u32>(bytes).unwrap();
    assert_eq!(num, 123);
}

#[test]
fn test_as_number_invalid() {
    let bytes = "12 3".to_string().into_bytes();
    let err = as_number::<u32>(bytes).unwrap_err();
    assert_eq!(err, TcpTransportError::NumberParseError);
}


// Basic methods to consume the stream

#[test]
fn test_read_bytes() {
    // "a\r\n"
    let ts = TestStream::new(vec![93, 13, 10]);
    let mut transport = TcpTransport::new(ts);

    let bytes = transport.read_bytes_exact(3).unwrap();
    assert_eq!(bytes, [93, 13, 10]);
}

#[test]
fn test_read_bytes_too_few() {
    // "a"
    let ts = TestStream::new(vec![93]);
    let mut transport = TcpTransport::new(ts);

    let bytes = transport.read_bytes_exact(2).unwrap();
    assert_eq!(bytes, [93]);
}

#[test]
fn test_read_bytes_many() {
    // "a" * 1mb
    let ts = TestStream::new(vec![93; 1 << 20]);
    let mut transport = TcpTransport::new(ts);

    let bytes = transport.read_bytes_exact(1 << 20).unwrap();
    assert_eq!(bytes, vec![93; 1 << 20]);
}


#[test]
fn test_read_word_in_line_one_char() {
    // "a a"
    let ts = TestStream::new(vec![93, 32, 93]);
    let mut transport = TcpTransport::new(ts);

    let (word, eol) = transport.read_word_in_line().unwrap();
    assert_eq!(word, &[93]);
    assert_eq!(false, eol);
}

#[test]
fn test_read_word_in_line_leading_spaces() {
    // "  a "
    let ts = TestStream::new(vec![32, 32, 93, 32]);
    let mut transport = TcpTransport::new(ts);

    let (word, eol) = transport.read_word_in_line().unwrap();
    assert_eq!(word, &[93]);
    assert_eq!(false, eol);
}

#[test]
fn test_read_word_in_line_eol() {
    // "\r\n"
    let ts = TestStream::new(vec![13, 10]);
    let mut transport = TcpTransport::new(ts);

    let (word, eol) = transport.read_word_in_line().unwrap();
    assert_eq!(word, &[]);
    assert_eq!(true, eol);
}


#[test]
fn test_read_line_as_words_ok() {
    // "aa bb\r\n"
    let ts = TestStream::new(vec![93, 93, 32, 32, 94, 94, 13, 10]);
    let mut transport = TcpTransport::new(ts);

    let words = transport.read_line_as_words().unwrap();
    assert_eq!(words, &[&[93, 93], &[94, 94]]);
}

#[test]
fn test_read_line_as_words_surrounding_space() {
    // "  a  b  \r\n"
    let ts = TestStream::new(vec![32, 32, 93, 32, 32, 94, 32, 32, 13, 10]);
    let mut transport = TcpTransport::new(ts);

    let words = transport.read_line_as_words().unwrap();
    assert_eq!(words, &[&[93], &[94]]);
}


// Basic methods to produce the stream

#[test]
fn test_write_bytes() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let bytelen = transport.write_bytes(&vec![97, 98, 99]).unwrap();
    assert_eq!(3, bytelen);
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


// Command parsing: Delete

#[test]
fn test_read_cmd_delete() {
    let cmd_str = "delete x \r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Delete(Delete::new("x", false)));
}

#[test]
fn test_read_cmd_delete_noreply() {
    let cmd_str = "delete x noreply\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Delete(Delete::new("x", true)));
}


// Command parsing: Get

#[test]
fn test_read_cmd_get_one_key() {
    let cmd_str = "get x\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Get(Get::one("x")));
}

#[test]
fn test_read_cmd_get_two_keys() {
    let cmd_str = "get x y\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let keys = vec!["x".to_string(), "y".to_string()];
    assert_eq!(cmd, Cmd::Get(Get::new(keys)));
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
    let cmd_str = "set x 15 0 3 \r\nabc\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let exp = Set::new(SetInstr::Set, "x", 15, 0, vec![97, 98, 99], false);
    assert_eq!(cmd, Cmd::Set(exp));
}

#[test]
fn test_read_cmd_set_noreply_ok() {
    let cmd_str = "set x 15 0 3 noreply\r\nabc\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let exp = Set::new(SetInstr::Set, "x", 15, 0, vec![97, 98, 99], true);
    assert_eq!(cmd, Cmd::Set(exp));
}

#[test]
fn test_read_cmd_set_under_size() {
    let cmd_str = "set x 0 0 2 \r\nabc\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_cmd().unwrap_err();
    assert_eq!(err, TcpTransportError::CommandParseError);
}

#[test]
fn test_read_cmd_set_over_size() {
    let cmd_str = "set x 0 0 4 \r\nabc\r\n".to_string();
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
    }

    // Test for truncated stream
    try_cmd("set x 0 0 3 \r\nabc\r");
    try_cmd("set x 0 0 3 \r\nabc");
    try_cmd("set x 0 0 3 \r\nab");
    try_cmd("set x 0 0 3 \r\na");
    try_cmd("set x 0 0 3 \r\n");
    try_cmd("set x 0 0 3 \r");
    try_cmd("set x 0 0 3 ");
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


// Response writing: Deleted

#[test]
fn test_write_resp_deleted() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Deleted;
    transport.write_resp(&resp).unwrap();
    let expected = "DELETED\r\n".to_string().into_bytes();
    assert_eq!(transport.get_stream().outgoing, expected);
}


// Response writing: Empty

#[test]
fn test_write_resp_empty() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Empty;
    transport.write_resp(&resp).unwrap();
    let expected = "".to_string().into_bytes();
    assert_eq!(transport.get_stream().outgoing, expected);
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


// Response writing: NotFound

#[test]
fn test_write_resp_not_found() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::NotFound;
    transport.write_resp(&resp).unwrap();
    let expected = "NOT_FOUND\r\n".to_string().into_bytes();
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
fn test_write_resp_value_one() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let val1 = Value::new("x", 15, "abc".to_string().into_bytes());
    let resp = Resp::Values(vec![val1]);
    transport.write_resp(&resp).unwrap();
    let expected = "VALUE x 15 3\r\nabc\r\nEND\r\n";
    let exp_bytes = expected.to_string().into_bytes();
    assert_eq!(transport.get_stream().outgoing, exp_bytes);
}

#[test]
fn test_write_resp_value_two() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let val1 = Value::new("x", 15, "abc".to_string().into_bytes());
    let val2 = Value::new("y", 17, "def".to_string().into_bytes());
    let resp = Resp::Values(vec![val1, val2]);
    transport.write_resp(&resp).unwrap();
    let expected = "VALUE x 15 3\r\nabc\r\nVALUE y 17 3\r\ndef\r\nEND\r\n";
    let exp_bytes = expected.to_string().into_bytes();
    assert_eq!(transport.get_stream().outgoing, exp_bytes);
}
