use storage::Cache;

use super::Driver;
use super::cmd::Set;
use super::cmd::Get;
use super::cmd::Cmd;
use super::cmd::Resp;
use super::cmd::Value;


// Deconstruct Resp to a Value, panic if unsuccessful
fn get_resp_value(resp: Resp) -> Value {
    match resp {
        Resp::Value(value) => value,
        _ => {
            panic!("Could not match Resp::Value");
        }
    }
}


#[test]
fn test_set_and_get_a_key() {
    let mut cache = Cache::with_defaults(100);
    let mut driver = Driver::new(cache);

    let key_name = "x";
    let blob = vec![1, 2, 3];

    // Set a key
    let set = Set {
        key: key_name.to_string(),
        exptime: 0,
        data: blob.clone(),
    };
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Retrieve it
    let get = Get { key: key_name.to_string() };
    let cmd = Cmd::Get(get);
    let resp = driver.run(cmd);
    assert_eq!(blob, get_resp_value(resp).data);

    // Try to retrieve a key not set
    let get = Get { key: "y".to_string() };
    let cmd = Cmd::Get(get);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Error);
}
