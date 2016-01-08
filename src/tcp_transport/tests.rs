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
    let bytes = b"123".to_vec();
    let num = as_number::<u32>(bytes).unwrap();
    assert_eq!(num, 123);
}

#[test]
fn test_as_number_invalid() {
    let bytes = b"12 3".to_vec();
    let err = as_number::<u32>(bytes).unwrap_err();
    assert_eq!(err, TcpTransportError::NumberParseError);
}


// Basic methods to consume the stream

#[test]
fn test_read_bytes() {
    let ts = TestStream::new(vec![97, 13, 10]);
    let mut transport = TcpTransport::new(ts);

    let bytes = transport.read_bytes_exact(3).unwrap();
    assert_eq!(bytes, b"a\r\n");
}

#[test]
fn test_read_bytes_too_few() {
    let ts = TestStream::new(vec![97]);
    let mut transport = TcpTransport::new(ts);

    let bytes = transport.read_bytes_exact(2).unwrap();
    assert_eq!(bytes, b"a");
}

#[test]
fn test_read_bytes_many() {
    // "a" * 1mb
    let ts = TestStream::new(vec![97; 1 << 20]);
    let mut transport = TcpTransport::new(ts);

    let bytes = transport.read_bytes_exact(1 << 20).unwrap();
    assert_eq!(bytes, vec![97; 1 << 20]);
}


#[test]
fn test_read_word_in_line_one_char() {
    let ts = TestStream::new(b"a a".to_vec());
    let mut transport = TcpTransport::new(ts);

    let (word, eol) = transport.read_word_in_line().unwrap();
    assert_eq!(word, b"a");
    assert_eq!(false, eol);
}

#[test]
fn test_read_word_in_line_leading_spaces() {
    let ts = TestStream::new(b"  a ".to_vec());
    let mut transport = TcpTransport::new(ts);

    let (word, eol) = transport.read_word_in_line().unwrap();
    assert_eq!(word, b"a");
    assert_eq!(false, eol);
}

#[test]
fn test_read_word_in_line_eol() {
    let ts = TestStream::new(b"\r\n".to_vec());
    let mut transport = TcpTransport::new(ts);

    let (word, eol) = transport.read_word_in_line().unwrap();
    assert_eq!(word, b"");
    assert_eq!(true, eol);
}


#[test]
fn test_read_line_as_words_ok() {
    let ts = TestStream::new(b"aa bb\r\n".to_vec());
    let mut transport = TcpTransport::new(ts);

    let words = transport.read_line_as_words().unwrap();
    assert_eq!(words, &[b"aa", b"bb"]);
}

#[test]
fn test_read_line_as_words_surrounding_space() {
    let ts = TestStream::new(b"  a  b  \r\n".to_vec());
    let mut transport = TcpTransport::new(ts);

    let words = transport.read_line_as_words().unwrap();
    assert_eq!(words, &[b"a", b"b"]);
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
    let cmd_str = b"invalid key 0 0 3\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_cmd().unwrap_err();
    assert_eq!(err, TcpTransportError::InvalidCmd);
}

#[test]
fn test_read_cmd_malterminated() {
    let cmd_str = b"stats\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_cmd().unwrap_err();
    assert_eq!(err, TcpTransportError::StreamReadError);
}


// Command parsing: Add

#[test]
fn test_read_cmd_add() {
    let cmd_str = b"add x 15 0 3 \r\nabc\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let exp = Set::new(SetInstr::Add, "x", 15, 0, vec![97, 98, 99], false);
    assert_eq!(cmd, Cmd::Set(exp));
}


// Command parsing: Append

#[test]
fn test_read_cmd_append() {
    let cmd_str = b"append x 15 0 3 \r\nabc\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let exp = Set::new(SetInstr::Append, "x", 15, 0, vec![97, 98, 99], false);
    assert_eq!(cmd, Cmd::Set(exp));
}


// Command parsing: Cas

#[test]
fn test_read_cmd_cas() {
    let cmd_str = b"cas x 15 0 3 44 \r\nabc\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let mut exp = Set::new(SetInstr::Cas, "x", 15, 0, vec![97, 98, 99], false);
    exp.with_cas_unique(44);
    assert_eq!(cmd, Cmd::Set(exp));
}

#[test]
fn test_read_cmd_cas_noreply() {
    let cmd_str = b"cas x 15 0 3 44 noreply\r\nabc\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let mut exp = Set::new(SetInstr::Cas, "x", 15, 0, vec![97, 98, 99], true);
    exp.with_cas_unique(44);
    assert_eq!(cmd, Cmd::Set(exp));
}


// Command parsing: Decr

#[test]
fn test_read_cmd_decr() {
    let cmd_str = b"decr x 5 \r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Inc(Inc::new(IncInstr::Decr, "x", 5, false)));
}

#[test]
fn test_read_cmd_decr_noreply() {
    let cmd_str = b"decr x 5 noreply\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Inc(Inc::new(IncInstr::Decr, "x", 5, true)));
}


// Command parsing: Delete

#[test]
fn test_read_cmd_delete() {
    let cmd_str = b"delete x \r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Delete(Delete::new("x", false)));
}

#[test]
fn test_read_cmd_delete_noreply() {
    let cmd_str = b"delete x noreply\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Delete(Delete::new("x", true)));
}


// Command parsing: FlushAll

#[test]
fn test_read_cmd_flush_all() {
    let cmd_str = b"flush_all\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::FlushAll(FlushAll::new(None, false)));
}


// Command parsing: Get

#[test]
fn test_read_cmd_get_one_key() {
    let cmd_str = b"get x\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Get(Get::one(GetInstr::Get, "x")));
}

#[test]
fn test_read_cmd_get_two_keys() {
    let cmd_str = b"get x y\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let keys = vec!["x".to_string(), "y".to_string()];
    assert_eq!(cmd, Cmd::Get(Get::new(GetInstr::Get, keys)));
}

#[test]
fn test_read_cmd_get_non_utf8() {
    let cmd_bytes = b"get \xfe\r\n".to_vec();
    let ts = TestStream::new(cmd_bytes);
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_cmd().unwrap_err();
    assert_eq!(err, TcpTransportError::Utf8Error);
}

#[test]
fn test_read_cmd_get_malformed() {
    fn try_cmd(cmd: &[u8]) {
        let ts = TestStream::new(cmd.to_vec());
        let mut transport = TcpTransport::new(ts);

        let err = transport.read_cmd().unwrap_err();
        assert_eq!(err, TcpTransportError::StreamReadError);
    }

    // Test for truncated stream
    try_cmd(b"get x\r");
    try_cmd(b"get x");
    try_cmd(b"get ");
    try_cmd(b"get");
}


// Command parsing: Gets

#[test]
fn test_read_cmd_gets_one_key() {
    let cmd_str = b"gets x\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Get(Get::one(GetInstr::Gets, "x")));
}


// Command parsing: Incr

#[test]
fn test_read_cmd_incr() {
    let cmd_str = b"incr x 5 \r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Inc(Inc::new(IncInstr::Incr, "x", 5, false)));
}

#[test]
fn test_read_cmd_incr_noreply() {
    let cmd_str = b"incr x 5 noreply\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Inc(Inc::new(IncInstr::Incr, "x", 5, true)));
}


// Command parsing: Quit

#[test]
fn test_read_cmd_quit() {
    let cmd_str = b"quit\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Quit);
}


// Command parsing: Prepend

#[test]
fn test_read_cmd_prepend() {
    let cmd_str = b"prepend x 15 0 3 \r\nabc\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let exp = Set::new(SetInstr::Prepend, "x", 15, 0, vec![97, 98, 99], false);
    assert_eq!(cmd, Cmd::Set(exp));
}


// Command parsing: Replace

#[test]
fn test_read_cmd_replace() {
    let cmd_str = b"replace x 15 0 3 \r\nabc\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let exp = Set::new(SetInstr::Replace, "x", 15, 0, vec![97, 98, 99], false);
    assert_eq!(cmd, Cmd::Set(exp));
}


// Command parsing: Set

#[test]
fn test_read_cmd_set_ok() {
    let cmd_str = b"set x 15 0 3 \r\nabc\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let exp = Set::new(SetInstr::Set, "x", 15, 0, vec![97, 98, 99], false);
    assert_eq!(cmd, Cmd::Set(exp));
}

#[test]
fn test_read_cmd_set_noreply_ok() {
    let cmd_str = b"set x 15 0 3 noreply\r\nabc\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let exp = Set::new(SetInstr::Set, "x", 15, 0, vec![97, 98, 99], true);
    assert_eq!(cmd, Cmd::Set(exp));
}

#[test]
fn test_read_cmd_set_under_size() {
    let cmd_str = b"set x 0 0 2 \r\nabc\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_cmd().unwrap_err();
    assert_eq!(err, TcpTransportError::CommandParseError);
}

#[test]
fn test_read_cmd_set_over_size() {
    let cmd_str = b"set x 0 0 4 \r\nabc\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let err = transport.read_cmd().unwrap_err();
    assert_eq!(err, TcpTransportError::CommandParseError);
}

#[test]
fn test_read_cmd_set_malformed() {
    fn try_cmd(cmd: &[u8]) {
        let ts = TestStream::new(cmd.to_vec());
        let mut transport = TcpTransport::new(ts);

        transport.read_cmd().unwrap_err();
    }

    // Test for truncated stream
    try_cmd(b"set x 0 0 3 \r\nabc\r");
    try_cmd(b"set x 0 0 3 \r\nabc");
    try_cmd(b"set x 0 0 3 \r\nab");
    try_cmd(b"set x 0 0 3 \r\na");
    try_cmd(b"set x 0 0 3 \r\n");
    try_cmd(b"set x 0 0 3 \r");
    try_cmd(b"set x 0 0 3 ");
    try_cmd(b"set x 0 0 3");
    try_cmd(b"set x 0 0 ");
    try_cmd(b"set x 0 0");
    try_cmd(b"set x 0 ");
    try_cmd(b"set x 0");
    try_cmd(b"set x ");
    try_cmd(b"set x");
    try_cmd(b"set ");
    try_cmd(b"set");
}


// Command parsing: Stats

#[test]
fn test_read_cmd_stats() {
    let cmd_str = b"stats\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    assert_eq!(cmd, Cmd::Stats);
}


// Command parsing: Touch

#[test]
fn test_read_cmd_touch() {
    let cmd_str = b"touch x 0 \r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let touch = Touch::new("x", 0, false);
    assert_eq!(cmd, Cmd::Touch(touch));
}

#[test]
fn test_read_cmd_touch_noreply() {
    let cmd_str = b"touch x 0 noreply\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
    let mut transport = TcpTransport::new(ts);

    let cmd = transport.read_cmd().unwrap();
    let touch = Touch::new("x", 0, true);
    assert_eq!(cmd, Cmd::Touch(touch));
}


// Command parsing: Version

#[test]
fn test_read_cmd_version() {
    let cmd_str = b"version\r\n".to_vec();
    let ts = TestStream::new(cmd_str);
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
    let expected = b"CLIENT_ERROR woops my bad\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}


// Response writing: Deleted

#[test]
fn test_write_resp_deleted() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Deleted;
    transport.write_resp(&resp).unwrap();
    let expected = b"DELETED\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}


// Response writing: Empty

#[test]
fn test_write_resp_empty() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Empty;
    transport.write_resp(&resp).unwrap();
    let expected = b"";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}


// Response writing: Error

#[test]
fn test_write_resp_error() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Error;
    transport.write_resp(&resp).unwrap();
    let expected = b"ERROR\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}


// Response writing: Exists

#[test]
fn test_write_resp_exists() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Exists;
    transport.write_resp(&resp).unwrap();
    let expected = b"EXISTS\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}


// Response writing: IntValue

#[test]
fn test_write_resp_intvalue() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::IntValue(5);
    transport.write_resp(&resp).unwrap();
    let expected = b"5\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}


// Response writing: NotFound

#[test]
fn test_write_resp_not_found() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::NotFound;
    transport.write_resp(&resp).unwrap();
    let expected = b"NOT_FOUND\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}


// Response writing: NotStored

#[test]
fn test_write_resp_not_stored() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::NotStored;
    transport.write_resp(&resp).unwrap();
    let expected = b"NOT_STORED\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}


// Response writing: Ok

#[test]
fn test_write_resp_ok() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Ok;
    transport.write_resp(&resp).unwrap();
    let expected = b"OK\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}


// Response writing: ServerError

#[test]
fn test_write_resp_servererror() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::ServerError("woops my bad".to_string());
    transport.write_resp(&resp).unwrap();
    let expected = b"SERVER_ERROR woops my bad\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}


// Response writing: Stats

#[test]
fn test_write_resp_stats() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let stat = Stat::new("curr_items", "0".to_string());
    let resp = Resp::Stats(vec![stat]);
    transport.write_resp(&resp).unwrap();
    let expected = b"STAT curr_items 0\r\nEND\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}


// Response writing: Stored

#[test]
fn test_write_resp_stored() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Stored;
    transport.write_resp(&resp).unwrap();
    let expected = b"STORED\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}


// Response writing: Touched

#[test]
fn test_write_resp_touched() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Touched;
    transport.write_resp(&resp).unwrap();
    let expected = b"TOUCHED\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}


// Response writing: Value

#[test]
fn test_write_resp_value_one() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let val1 = Value::new("x", 15, b"abc".to_vec());
    let resp = Resp::Values(vec![val1]);
    transport.write_resp(&resp).unwrap();
    let expected = b"VALUE x 15 3\r\nabc\r\nEND\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}

#[test]
fn test_write_resp_value_two() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let val1 = Value::new("x", 15, b"abc".to_vec());
    let val2 = Value::new("y", 17, b"def".to_vec());
    let resp = Resp::Values(vec![val1, val2]);
    transport.write_resp(&resp).unwrap();
    let expected = b"VALUE x 15 3\r\nabc\r\nVALUE y 17 3\r\ndef\r\nEND\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}

#[test]
fn test_write_resp_value_one_cas() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let mut val1 = Value::new("x", 15, b"abc".to_vec());
    val1.with_cas_unique(45);
    let resp = Resp::Values(vec![val1]);
    transport.write_resp(&resp).unwrap();
    let expected = b"VALUE x 15 3 45\r\nabc\r\nEND\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}


// Response writing: Version

#[test]
fn test_write_resp_version() {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    let resp = Resp::Version("1.0.1".to_string());
    transport.write_resp(&resp).unwrap();
    let expected = b"VERSION 1.0.1\r\n";
    assert_eq!(transport.get_stream().outgoing, expected.to_vec());
}
