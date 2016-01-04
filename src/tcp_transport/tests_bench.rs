use test::Bencher;

use protocol::cmd::Cmd;
use protocol::cmd::Get;
use protocol::cmd::Resp;
use protocol::cmd::Set;
use protocol::cmd::SetInstr;
use protocol::cmd::Value;
use testlib::test_stream::TestStream;

use super::TcpTransport;


// Reading

use platform::time::sleep_secs;
#[bench]
fn bench_transport_read_cmd_get(b: &mut Bencher) {
    let cmd_str = "get variable1 variable2\r\n".to_string().into_bytes();
    let mut ts = TestStream::new(vec![]);
    ts.set_incoming_rep(cmd_str);  // set stream to repeating mode
    let mut transport = TcpTransport::new(ts);

    b.iter(|| transport.read_cmd().unwrap())
}

#[bench]
fn bench_transport_read_cmd_set(b: &mut Bencher) {
    let cmd_str = "set variable 13 1 10 noreply\r\n0123456789\r\n"
                      .to_string()
                      .into_bytes();
    let mut ts = TestStream::new(vec![]);
    ts.set_incoming_rep(cmd_str);  // set stream to repeating mode
    let mut transport = TcpTransport::new(ts);

    b.iter(|| {
        transport.read_cmd().unwrap();
    })
}


// Writing

#[bench]
fn bench_transport_write_resp_value(b: &mut Bencher) {
    let ts = TestStream::new(vec![]);
    let mut transport = TcpTransport::new(ts);

    b.iter(|| {
        let val = Value::new("x", 15, "abc".to_string().into_bytes());
        let resp = Resp::Values(vec![val]);
        transport.write_resp(&resp).unwrap();
    })
}
