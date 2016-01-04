use common::consts::get_version_string;
use platform::process::get_pid;
use platform::time::sleep_secs;
use platform::time::time_now;
use storage::Cache;

use super::Driver;
use super::cmd::Cmd;
use super::cmd::Delete;
use super::cmd::Get;
use super::cmd::Resp;
use super::cmd::Set;
use super::cmd::SetInstr;
use super::cmd::Stat;
use super::cmd::Value;


// Add

#[test]
fn test_cmd_add() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    // Add a new key
    let set = Set::new(SetInstr::Add, "x", 4, 0, vec![8, 9], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Make sure it was added
    let cmd = Cmd::Get(Get::one("x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![8, 9], resp.get_first_value().unwrap().data);
    assert_eq!(4, resp.get_first_value().unwrap().flags);

    // Try using add to overwrite an existing key
    let set = Set::new(SetInstr::Add, "x", 5, 0, vec![11], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::NotStored);

    // Make sure it was not overwritten
    let cmd = Cmd::Get(Get::one("x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![8, 9], resp.get_first_value().unwrap().data);
    assert_eq!(4, resp.get_first_value().unwrap().flags);

    // Add with noreply
    let set = Set::new(SetInstr::Add, "y", 5, 0, vec![11], true);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Make sure it was added
    let cmd = Cmd::Get(Get::one("y"));
    let resp = driver.run(cmd);
    assert_eq!(vec![11], resp.get_first_value().unwrap().data);
    assert_eq!(5, resp.get_first_value().unwrap().flags);
}


// Append

#[test]
fn test_cmd_append() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    // Try to append to an invalid key
    let set = Set::new(SetInstr::Append, "x", 4, 0, vec![8, 9], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Error);

    // Set a key we can append to
    let set = Set::new(SetInstr::Set, "x", 0, 0, vec![8, 9], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Append to it
    let set = Set::new(SetInstr::Append, "x", 4, 0, vec![10], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Make sure it was updated
    let cmd = Cmd::Get(Get::one("x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![8, 9, 10], resp.get_first_value().unwrap().data);
    assert_eq!(4, resp.get_first_value().unwrap().flags);

    // Append again, in noreply mode
    let set = Set::new(SetInstr::Append, "x", 5, 0, vec![11], true);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Make sure it was updated again
    let cmd = Cmd::Get(Get::one("x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![8, 9, 10, 11], resp.get_first_value().unwrap().data);
    assert_eq!(5, resp.get_first_value().unwrap().flags);
}


// Delete

#[test]
fn test_cmd_delete() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    let key_name = "x";
    let blob = vec![1, 2, 3];

    // Try to delete a key that does not exist
    let cmd = Cmd::Delete(Delete::new("z", false));
    let resp = driver.run(cmd);
    assert_eq!(Resp::NotFound, resp);

    // Again, but now with noreply flag
    let cmd = Cmd::Delete(Delete::new("z", true));
    let resp = driver.run(cmd);
    assert_eq!(Resp::Empty, resp);

    // Set a key we can delete later
    let set = Set::new(SetInstr::Set, "x", 0, 0, vec![8, 9], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // And another
    let set = Set::new(SetInstr::Set, "y", 0, 0, vec![8, 9], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Delete the first one
    let cmd = Cmd::Delete(Delete::new("x", false));
    let resp = driver.run(cmd);
    assert_eq!(Resp::Deleted, resp);

    // Make sure it's gone
    let cmd = Cmd::Get(Get::one("x"));
    let resp = driver.run(cmd);
    assert_eq!(0, resp.get_values().unwrap().len());

    // Delete the second - with noreply
    let cmd = Cmd::Delete(Delete::new("y", true));
    let resp = driver.run(cmd);
    assert_eq!(Resp::Empty, resp);

    // Make sure it's gone
    let cmd = Cmd::Get(Get::one("y"));
    let resp = driver.run(cmd);
    assert_eq!(0, resp.get_values().unwrap().len());
}


// Get and Set

#[test]
fn test_cmd_set_and_get_a_key() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    let key_name = "x";
    let blob = vec![1, 2, 3];

    // Try to retrieve a key not set
    let cmd = Cmd::Get(Get::one(key_name));
    let resp = driver.run(cmd);
    assert_eq!(0, resp.get_values().unwrap().len());

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

    // Set a key with noreply flag
    let set = Set::new(SetInstr::Set, "y", 15, 0, blob.clone(), true);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Retrieve it
    let cmd = Cmd::Get(Get::one("y"));
    let resp = driver.run(cmd);
    assert_eq!(15, resp.get_first_value().unwrap().flags);
    assert_eq!(blob, resp.get_first_value().unwrap().data);
}

#[test]
fn test_cmd_set_and_get_multiple_keys() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    let val1 = vec![1];
    let val3 = vec![3];

    // Set two keys
    let set = Set::new(SetInstr::Set, "a", 15, 0, val1.clone(), false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    let set = Set::new(SetInstr::Set, "c", 17, 0, val3.clone(), false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Try to retrieve three keys - get two
    let keys = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let cmd = Cmd::Get(Get::new(keys));
    let resp = driver.run(cmd);

    let values = resp.get_values().unwrap();
    let val1 = Value::new("a", 15, val1);
    let val3 = Value::new("c", 17, val3);
    assert_eq!(2, values.len());
    assert_eq!(val1, values[0]);
    assert_eq!(val3, values[1]);
}


// Replace

#[test]
fn test_cmd_replace() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    // Try to replace an non-existent key
    let set = Set::new(SetInstr::Replace, "x", 0, 0, vec![8, 9], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::NotStored);

    // Set a key
    let set = Set::new(SetInstr::Set, "x", 0, 0, vec![8, 9], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Replace a valid key
    let set = Set::new(SetInstr::Replace, "x", 4, 0, vec![10], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Make sure it was updated
    let cmd = Cmd::Get(Get::one("x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![10], resp.get_first_value().unwrap().data);
    assert_eq!(4, resp.get_first_value().unwrap().flags);

    // Replace a valid key in noreply mode
    let set = Set::new(SetInstr::Replace, "x", 6, 0, vec![11], true);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Make sure it was updated
    let cmd = Cmd::Get(Get::one("x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![11], resp.get_first_value().unwrap().data);
    assert_eq!(6, resp.get_first_value().unwrap().flags);
}


// Stats

#[test]
fn test_cmd_stats() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    // Set a key
    let set = Set::new(SetInstr::Set, "x", 0, 0, vec![8, 9], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Retrieve it
    let cmd = Cmd::Get(Get::one("x"));
    driver.run(cmd);

    // Run stats
    let cmd = Cmd::Stats;
    let resp = driver.run(cmd);

    let st_pid = Stat::new("pid", get_pid().to_string());
    let st_bytes = Stat::new("bytes", "3".to_string());
    let st_uptime = Stat::new("uptime", "0".to_string());
    let st_time = Stat::new("time", (time_now() as u64).to_string());
    let st_version = Stat::new("version", get_version_string());
    let st_cmd_get = Stat::new("cmd_get", "1".to_string());
    let st_cmd_set = Stat::new("cmd_set", "1".to_string());
    let st_cmd_flush = Stat::new("cmd_flush", "0".to_string());
    let st_cmd_touch = Stat::new("cmd_touch", "0".to_string());
    let st_get_hits = Stat::new("get_hits", "1".to_string());
    let st_get_misses = Stat::new("get_misses", "0".to_string());
    let st_delete_hits = Stat::new("delete_hits", "0".to_string());
    let st_delete_misses = Stat::new("delete_misses", "0".to_string());
    let st_bytes_read = Stat::new("bytes_read", "0".to_string());
    let st_bytes_written = Stat::new("bytes_written", "0".to_string());
    let st_limit_maxbytes = Stat::new("limit_maxbytes", "100".to_string());
    let st_curr_items = Stat::new("curr_items", "1".to_string());
    let st_total_items = Stat::new("total_items", "1".to_string());
    let st_evictions = Stat::new("evictions", "0".to_string());
    let st_reclaimed = Stat::new("reclaimed", "0".to_string());

    let stats = resp.get_stats().unwrap();
    assert_eq!(*stats,
               (vec![st_pid,
                     st_uptime,
                     st_time,
                     st_version,
                     st_cmd_get,
                     st_cmd_set,
                     st_cmd_flush,
                     st_cmd_touch,
                     st_get_hits,
                     st_get_misses,
                     st_delete_hits,
                     st_delete_misses,
                     st_bytes_read,
                     st_bytes_written,
                     st_limit_maxbytes,
                     st_bytes,
                     st_curr_items,
                     st_total_items,
                     st_evictions,
                     st_reclaimed]));
}


// Version

#[test]
fn test_cmd_version() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    // Set a key
    let cmd = Cmd::Version;
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Version(get_version_string()));
}


// Item expiration cases

// this is a slow test that relies on sleeps
#[ignore]
#[test]
fn test_cmd_relative_exptime() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    let key_name = "x";
    let blob = vec![1, 2, 3];

    // Set a key with exptime of 1 second
    let set = Set::new(SetInstr::Set, key_name, 0, 1, blob.clone(), false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Retrieve it right away - succeeds
    let cmd = Cmd::Get(Get::one(key_name));
    let resp = driver.run(cmd);
    assert_eq!(blob, resp.get_first_value().unwrap().data);

    // sleep 1.5 secs - long enough to expire key
    sleep_secs(1.5);

    // Retrieve the key again - it's gone
    let cmd = Cmd::Get(Get::one(key_name));
    let resp = driver.run(cmd);
    assert_eq!(0, resp.get_values().unwrap().len());
}

// this is a slow test that relies on sleeps
#[ignore]
#[test]
fn test_cmd_absolute_exptime() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    let key_name = "x";
    let blob = vec![1, 2, 3];
    let exp = time_now().round() as u32 + 1;

    // Set a key with exptime of 1 second
    let set = Set::new(SetInstr::Set, key_name, 0, exp, blob.clone(), false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Retrieve it right away - succeeds
    let cmd = Cmd::Get(Get::one(key_name));
    let resp = driver.run(cmd);
    assert_eq!(blob, resp.get_first_value().unwrap().data);

    // sleep 2.5 secs - long enough to expire key
    sleep_secs(2.5);

    // Retrieve the key again - it's gone
    let cmd = Cmd::Get(Get::one(key_name));
    let resp = driver.run(cmd);
    assert_eq!(0, resp.get_values().unwrap().len());
}
