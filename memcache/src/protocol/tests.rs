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
    let cmd = Cmd::Set(Set::new(key_name, 0, blob.clone()));
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Retrieve it
    let cmd = Cmd::Get(Get::new(key_name));
    let resp = driver.run(cmd);
    assert_eq!(blob, get_resp_value(resp).data);

    // Try to retrieve a key not set
    let cmd = Cmd::Get(Get::new("y"));
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Error);
}
