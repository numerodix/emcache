use test::Bencher;

use storage::Cache;

use super::Driver;
use super::cmd::Cmd;
use super::cmd::Get;
use super::cmd::Resp;
use super::cmd::Set;
use super::cmd::SetInstr;


#[bench]
fn bench_cmd_set_and_get_a_key(b: &mut Bencher) {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    let key_name = "x";
    let blob = vec![1, 2, 3];

    b.iter(|| {
        // Set a key
        let set = Set::new(SetInstr::Set, key_name, 15, 0, blob.clone(), false);
        let cmd = Cmd::Set(set);
        let resp = driver.run(cmd);
        assert_eq!(resp, Resp::Stored);

        // Retrieve it
        let cmd = Cmd::Get(Get::one(key_name));
        let resp = driver.run(cmd);
        assert_eq!(15, resp.get_first_value().unwrap().flags);
        assert_eq!(blob, resp.get_first_value().unwrap().data);
    })
}
