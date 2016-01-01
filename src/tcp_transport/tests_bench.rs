use test::Bencher;

use protocol::cmd::Cmd;
use protocol::cmd::Get;
use protocol::cmd::Resp;
use protocol::cmd::Set;
use protocol::cmd::Stat;
use protocol::cmd::Value;
use testlib::test_stream::TestStream;

use super::TcpTransport;


// Reading

#[bench]
fn bench_transport_read_cmd_get(b: &mut Bencher) {
    b.iter(|| {
        let cmd_str = "get x\r\n".to_string();
        let ts = TestStream::new(cmd_str.into_bytes());
        let mut transport = TcpTransport::new(ts);

        let cmd = transport.read_cmd().unwrap();
        assert_eq!(cmd, Cmd::Get(Get::new("x")));
    })
}

#[bench]
fn bench_transport_read_cmd_set(b: &mut Bencher) {
    b.iter(|| {
        let cmd_str = "set x 0 0 3\r\nabc\r\n".to_string();
        let ts = TestStream::new(cmd_str.into_bytes());
        let mut transport = TcpTransport::new(ts);

        let cmd = transport.read_cmd().unwrap();
        assert_eq!(cmd, Cmd::Set(Set::new("x", 0, vec![97, 98, 99])));
    })
}


// Writing

#[bench]
fn bench_transport_write_resp_value(b: &mut Bencher) {
    b.iter(|| {
        let ts = TestStream::new(vec![]);
        let mut transport = TcpTransport::new(ts);

        let resp = Resp::Value(Value::new("x",
                                          "abc".to_string().into_bytes()));
        transport.write_resp(&resp).unwrap();
        let expected = "VALUE x 0 3\r\nabc\r\n".to_string().into_bytes();
        assert_eq!(transport.get_stream().outgoing, expected);
    })
}
