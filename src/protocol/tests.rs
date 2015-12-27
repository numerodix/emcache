use platform::process::get_pid;
use platform::time::sleep_secs;
use platform::time::time_now;
use storage::Cache;

use super::Driver;
use super::cmd::Cmd;
use super::cmd::Get;
use super::cmd::Resp;
use super::cmd::Set;
use super::cmd::Stat;
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
fn test_cmd_set_and_get_a_key() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    let key_name = "x";
    let blob = vec![1, 2, 3];

    // Try to retrieve a key not set
    let cmd = Cmd::Get(Get::new(key_name));
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Error);

    // Set a key
    let cmd = Cmd::Set(Set::new(key_name, 0, blob.clone()));
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Retrieve it
    let cmd = Cmd::Get(Get::new(key_name));
    let resp = driver.run(cmd);
    assert_eq!(blob, get_resp_value(resp).data);
}

#[test]
fn test_cmd_stats() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    // Set a key
    let cmd = Cmd::Set(Set::new("x", 0, vec![8, 9]));
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Retrieve it
    let cmd = Cmd::Get(Get::new("x"));
    let resp = driver.run(cmd);

    // Run stats
    let cmd = Cmd::Stats;
    let resp = driver.run(cmd);

    let st_pid = Stat::new("pid", get_pid().to_string());
    let st_bytes = Stat::new("bytes", "3".to_string());
    let st_uptime = Stat::new("uptime", "0".to_string());
    let st_cmd_get = Stat::new("cmd_get", "1".to_string());
    let st_cmd_set = Stat::new("cmd_set", "1".to_string());
    let st_cmd_flush = Stat::new("cmd_flush", "0".to_string());
    let st_cmd_touch = Stat::new("cmd_touch", "0".to_string());
    let st_time = Stat::new("time", (time_now() as u64).to_string());
    let st_curr_items = Stat::new("curr_items", "1".to_string());
    let st_total_items = Stat::new("total_items", "1".to_string());

    assert_eq!(resp,
               Resp::Stats(vec![st_pid,
                                st_uptime,
                                st_time,
                                st_cmd_get,
                                st_cmd_set,
                                st_cmd_flush,
                                st_cmd_touch,
                                st_bytes,
                                st_curr_items,
                                st_total_items]));
}

// this is a slow test that relies on sleeps
#[ignore]
#[test]
fn test_cmd_relative_exptime() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    let key_name = "x";
    let blob = vec![1, 2, 3];

    // Set a key with exptime of 1 second
    let cmd = Cmd::Set(Set::new(key_name, 1, blob.clone()));
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Retrieve it right away - succeeds
    let cmd = Cmd::Get(Get::new(key_name));
    let resp = driver.run(cmd);
    assert_eq!(blob, get_resp_value(resp).data);

    // sleep 1.5 secs - long enough to expire key
    sleep_secs(1.5);

    // Retrieve the key again - it's gone
    let cmd = Cmd::Get(Get::new(key_name));
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Error);
}

// this is a slow test that relies on sleeps
#[ignore]
#[test]
fn test_cmd_absolute_exptime() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    let key_name = "x";
    let blob = vec![1, 2, 3];
    let exptime = time_now().round() as u32 + 1;

    // Set a key with exptime of 1 second
    let cmd = Cmd::Set(Set::new(key_name, exptime, blob.clone()));
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Retrieve it right away - succeeds
    let cmd = Cmd::Get(Get::new(key_name));
    let resp = driver.run(cmd);
    assert_eq!(blob, get_resp_value(resp).data);

    // sleep 2.5 secs - long enough to expire key
    sleep_secs(2.5);

    // Retrieve the key again - it's gone
    let cmd = Cmd::Get(Get::new(key_name));
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Error);
}
