use protocol::cmd::Cmd;
use protocol::cmd::Delete;
use protocol::cmd::FlushAll;
use protocol::cmd::Get;
use protocol::cmd::GetInstr;
use protocol::cmd::Inc;
use protocol::cmd::IncInstr;
use protocol::cmd::Resp;
use protocol::cmd::Set;
use protocol::cmd::SetInstr;
use protocol::cmd::Stat;
use protocol::cmd::Touch;
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


// Command parsing: Add

#[test]
fn test_read_cmd_add() {
    let cmd_str = "add x 15 0 3 \r\nabc\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let exp = Set::new(SetInstr::Add, "x", 15, 0, vec![97, 98, 99], false);
    assert_eq!(cmd, Cmd::Set(exp));
}


// Command parsing: Append

#[test]
fn test_read_cmd_append() {
    let cmd_str = "append x 15 0 3 \r\nabc\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let exp = Set::new(SetInstr::Append, "x", 15, 0, vec![97, 98, 99], false);
    assert_eq!(cmd, Cmd::Set(exp));
}


// Command parsing: Cas

#[test]
fn test_read_cmd_cas() {
    let cmd_str = "cas x 15 0 3 44 \r\nabc\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let mut exp = Set::new(SetInstr::Cas, "x", 15, 0, vec![97, 98, 99], false);
    exp.with_cas_unique(44);
    assert_eq!(cmd, Cmd::Set(exp));
}

#[test]
fn test_read_cmd_cas_noreply() {
    let cmd_str = "cas x 15 0 3 44 noreply\r\nabc\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let mut exp = Set::new(SetInstr::Cas, "x", 15, 0, vec![97, 98, 99], true);
    exp.with_cas_unique(44);
    assert_eq!(cmd, Cmd::Set(exp));
}


// Command parsing: Decr

#[test]
fn test_read_cmd_decr() {
    let cmd_str = "decr x 5 \r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Inc(Inc::new(IncInstr::Decr, "x", 5, false)));
}

#[test]
fn test_read_cmd_decr_noreply() {
    let cmd_str = "decr x 5 noreply\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Inc(Inc::new(IncInstr::Decr, "x", 5, true)));
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


// Command parsing: FlushAll

#[test]
fn test_read_cmd_flush_all() {
    let cmd_str = "flush_all\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::FlushAll(FlushAll::new(None, false)));
}


// Command parsing: Get

#[test]
fn test_read_cmd_get_one_key() {
    let cmd_str = "get x\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Get(Get::one(GetInstr::Get, "x")));
}

#[test]
fn test_read_cmd_get_two_keys() {
    let cmd_str = "get x y\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let keys = vec!["x".to_string(), "y".to_string()];
    assert_eq!(cmd, Cmd::Get(Get::new(GetInstr::Get, keys)));
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


// Command parsing: Gets

#[test]
fn test_read_cmd_gets_one_key() {
    let cmd_str = "gets x\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Get(Get::one(GetInstr::Gets, "x")));
}


// Command parsing: Incr

#[test]
fn test_read_cmd_incr() {
    let cmd_str = "incr x 5 \r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Inc(Inc::new(IncInstr::Incr, "x", 5, false)));
}

#[test]
fn test_read_cmd_incr_noreply() {
    let cmd_str = "incr x 5 noreply\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Inc(Inc::new(IncInstr::Incr, "x", 5, true)));
}


// Command parsing: Quit

#[test]
fn test_read_cmd_quit() {
    let cmd_str = "quit\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Quit);
}


// Command parsing: Prepend

#[test]
fn test_read_cmd_prepend() {
    let cmd_str = "prepend x 15 0 3 \r\nabc\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let exp = Set::new(SetInstr::Prepend, "x", 15, 0, vec![97, 98, 99], false);
    assert_eq!(cmd, Cmd::Set(exp));
}


// Command parsing: Replace

#[test]
fn test_read_cmd_replace() {
    let cmd_str = "replace x 15 0 3 \r\nabc\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let exp = Set::new(SetInstr::Replace, "x", 15, 0, vec![97, 98, 99], false);
    assert_eq!(cmd, Cmd::Set(exp));
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


// Command parsing: Touch

#[test]
fn test_read_cmd_touch() {
    let cmd_str = "touch x 0 \r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let touch = Touch::new("x", 0, false);
    assert_eq!(cmd, Cmd::Touch(touch));
}

#[test]
fn test_read_cmd_touch_noreply() {
    let cmd_str = "touch x 0 noreply\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let touch = Touch::new("x", 0, true);
    assert_eq!(cmd, Cmd::Touch(touch));
}


// Command parsing: Version

#[test]
fn test_read_cmd_version() {
    let cmd_str = "version\r\n".to_string();
    let ts = TestStream::new(cmd_str.into_bytes());
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Version);
}


// Response writing: ClientError

#[test]
fn test_write_resp_clienterror() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::ClientError("woops my bad".to_string());
    transport.write_resp(&resp).unwrap();
    let expected = "CLIENT_ERROR woops my bad\r\n".to_string().into_bytes();
    assert_eq!(transport.get_stream().outgoing, expected);
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


// Response writing: IntValue

#[test]
fn test_write_resp_intvalue() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::IntValue(5);
    transport.write_resp(&resp).unwrap();
    let expected = "5\r\n".to_string().into_bytes();
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


// Response writing: NotStored

#[test]
fn test_write_resp_not_stored() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::NotStored;
    transport.write_resp(&resp).unwrap();
    let expected = "NOT_STORED\r\n".to_string().into_bytes();
    assert_eq!(transport.get_stream().outgoing, expected);
}


// Response writing: Ok

#[test]
fn test_write_resp_ok() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Ok;
    transport.write_resp(&resp).unwrap();
    let expected = "OK\r\n".to_string().into_bytes();
    assert_eq!(transport.get_stream().outgoing, expected);
}


// Response writing: ServerError

#[test]
fn test_write_resp_servererror() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::ServerError("woops my bad".to_string());
    transport.write_resp(&resp).unwrap();
    let expected = "SERVER_ERROR woops my bad\r\n".to_string().into_bytes();
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


// Response writing: Touched

#[test]
fn test_write_resp_touched() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Touched;
    transport.write_resp(&resp).unwrap();
    let expected = "TOUCHED\r\n".to_string().into_bytes();
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

#[test]
fn test_write_resp_value_one_cas() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let mut val1 = Value::new("x", 15, "abc".to_string().into_bytes());
    val1.with_cas_unique(45);
    let resp = Resp::Values(vec![val1]);
    transport.write_resp(&resp).unwrap();
    let expected = "VALUE x 15 3 45\r\nabc\r\nEND\r\n";
    let exp_bytes = expected.to_string().into_bytes();
    assert_eq!(transport.get_stream().outgoing, exp_bytes);
}


// Response writing: Version

#[test]
fn test_write_resp_version() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Version("1.0.1".to_string());
    transport.write_resp(&resp).unwrap();
    let expected = "VERSION 1.0.1\r\n".to_string().into_bytes();
    assert_eq!(transport.get_stream().outgoing, expected);
}
