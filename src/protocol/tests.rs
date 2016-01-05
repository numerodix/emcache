use common::consts::get_version_string;
use platform::process::get_pid;
use platform::time::sleep_secs;
use platform::time::time_now;
use storage::Cache;

use super::Driver;
use super::cmd::Cmd;
use super::cmd::Delete;
use super::cmd::FlushAll;
use super::cmd::Get;
use super::cmd::GetInstr;
use super::cmd::Inc;
use super::cmd::IncInstr;
use super::cmd::Resp;
use super::cmd::Set;
use super::cmd::SetInstr;
use super::cmd::Stat;
use super::cmd::Touch;
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
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![8, 9], resp.get_first_value().unwrap().data);
    assert_eq!(4, resp.get_first_value().unwrap().flags);

    // Try using add to overwrite an existing key
    let set = Set::new(SetInstr::Add, "x", 5, 0, vec![11], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::NotStored);

    // Try using add to overwrite an existing key - noreply
    let set = Set::new(SetInstr::Add, "x", 5, 0, vec![11], true);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Make sure it was not overwritten
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![8, 9], resp.get_first_value().unwrap().data);
    assert_eq!(4, resp.get_first_value().unwrap().flags);

    // Add with noreply
    let set = Set::new(SetInstr::Add, "y", 5, 0, vec![11], true);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Make sure it was added
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "y"));
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
    assert_eq!(resp, Resp::NotStored);

    // Try to append to an invalid key - noreply
    let set = Set::new(SetInstr::Append, "x", 4, 0, vec![8, 9], true);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

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
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![8, 9, 10], resp.get_first_value().unwrap().data);
    assert_eq!(4, resp.get_first_value().unwrap().flags);

    // Append again, in noreply mode
    let set = Set::new(SetInstr::Append, "x", 5, 0, vec![11], true);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Make sure it was updated again
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![8, 9, 10, 11], resp.get_first_value().unwrap().data);
    assert_eq!(5, resp.get_first_value().unwrap().flags);
}


// Cas

#[test]
fn test_cmd_cas() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    // Try to append to an invalid key
    let mut set = Set::new(SetInstr::Cas, "x", 4, 0, vec![8, 9], false);
    set.with_cas_unique(5);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::NotFound);

    // Try to append to an invalid key - noreply
    let mut set = Set::new(SetInstr::Cas, "x", 4, 0, vec![8, 9], true);
    set.with_cas_unique(5);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Set a key we can update
    let set = Set::new(SetInstr::Set, "x", 0, 0, vec![8, 9], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Obtain cas value
    let cmd = Cmd::Get(Get::one(GetInstr::Gets, "x"));
    let resp = driver.run(cmd);
    let cas_unique1 = resp.get_first_value().unwrap().cas_unique.unwrap();

    // Update it
    let mut set = Set::new(SetInstr::Cas, "x", 4, 0, vec![10], false);
    set.with_cas_unique(cas_unique1);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Make sure it was updated
    let cmd = Cmd::Get(Get::one(GetInstr::Gets, "x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![10], resp.get_first_value().unwrap().data);
    assert_eq!(4, resp.get_first_value().unwrap().flags);
    let cas_unique2 = resp.get_first_value().unwrap().cas_unique.unwrap();

    // Update it again - noreply
    let mut set = Set::new(SetInstr::Cas, "x", 7, 0, vec![11], true);
    set.with_cas_unique(cas_unique2);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Make sure it was updated
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![11], resp.get_first_value().unwrap().data);
    assert_eq!(7, resp.get_first_value().unwrap().flags);

    // Try to update it with a stale cas token
    let mut set = Set::new(SetInstr::Cas, "x", 4, 0, vec![10], false);
    set.with_cas_unique(cas_unique1);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Exists);
}


// Decr

#[test]
fn test_cmd_decr() {
    let cache = Cache::new(4096);
    let mut driver = Driver::new(cache);

    // Try to decr an invalid key
    let inc = Inc::new(IncInstr::Incr, "x", 4, false);
    let cmd = Cmd::Inc(inc);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::NotFound);

    // Try to decr an invalid key - noreply
    let inc = Inc::new(IncInstr::Decr, "x", 4, true);
    let cmd = Cmd::Inc(inc);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Set a key we can decr
    let set = Set::new(SetInstr::Set, "x", 0, 0, vec![b'2'], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Decr it
    let inc = Inc::new(IncInstr::Decr, "x", 1, false);
    let cmd = Cmd::Inc(inc);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::IntValue(1));

    // Make sure it was updated
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![b'1'], resp.get_first_value().unwrap().data);

    // Decr it again - noreply
    let inc = Inc::new(IncInstr::Decr, "x", 1, true);
    let cmd = Cmd::Inc(inc);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Make sure it was updated
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![b'0'], resp.get_first_value().unwrap().data);

    // Try to underflow it
    let inc = Inc::new(IncInstr::Decr, "x", 1, false);
    let cmd = Cmd::Inc(inc);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::IntValue(0));

    // Set a key we can't decr - would not fit in u64
    let set = Set::new(SetInstr::Set, "y", 0, 0, vec![b'1'; 255], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Try to decr it - fails
    let inc = Inc::new(IncInstr::Decr, "y", 1, false);
    let cmd = Cmd::Inc(inc);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::ClientError("Not a number".to_string()));
}


// Delete

#[test]
fn test_cmd_delete() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

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
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
    let resp = driver.run(cmd);
    assert_eq!(0, resp.get_values().unwrap().len());

    // Delete the second - with noreply
    let cmd = Cmd::Delete(Delete::new("y", true));
    let resp = driver.run(cmd);
    assert_eq!(Resp::Empty, resp);

    // Make sure it's gone
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "y"));
    let resp = driver.run(cmd);
    assert_eq!(0, resp.get_values().unwrap().len());
}


// FlushAll

#[test]
fn test_flush_all() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    let key_name = "x";
    let blob = vec![1, 2, 3];


    // Set a key
    let set = Set::new(SetInstr::Set, key_name, 15, 0, blob.clone(), false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Do a flush
    let cmd = Cmd::FlushAll(FlushAll::new(None, false));
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Ok);

    // The key is dead
    let cmd = Cmd::Get(Get::one(GetInstr::Get, key_name));
    let resp = driver.run(cmd);
    assert_eq!(0, resp.get_values().unwrap().len());


    // Set a key
    let set = Set::new(SetInstr::Set, key_name, 15, 0, blob.clone(), false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Do a flush - noreply
    let cmd = Cmd::FlushAll(FlushAll::new(None, true));
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // The key is dead
    let cmd = Cmd::Get(Get::one(GetInstr::Get, key_name));
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
    let cmd = Cmd::Get(Get::one(GetInstr::Get, key_name));
    let resp = driver.run(cmd);
    assert_eq!(0, resp.get_values().unwrap().len());

    // Set a key
    let set = Set::new(SetInstr::Set, key_name, 15, 0, blob.clone(), false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Retrieve it
    let cmd = Cmd::Get(Get::one(GetInstr::Get, key_name));
    let resp = driver.run(cmd);
    assert_eq!(15, resp.get_first_value().unwrap().flags);
    assert_eq!(blob, resp.get_first_value().unwrap().data);

    // Set a key with noreply flag
    let set = Set::new(SetInstr::Set, "y", 15, 0, blob.clone(), true);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Retrieve it
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "y"));
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
    let cmd = Cmd::Get(Get::new(GetInstr::Get, keys));
    let resp = driver.run(cmd);

    let values = resp.get_values().unwrap();
    let val1 = Value::new("a", 15, val1);
    let val3 = Value::new("c", 17, val3);
    assert_eq!(2, values.len());
    assert_eq!(val1, values[0]);
    assert_eq!(val3, values[1]);
}


// Gets

#[test]
fn get_cmd_gets() {
    let cache = Cache::new(4096);
    let mut driver = Driver::new(cache);

    // Set a key
    let set = Set::new(SetInstr::Set, "x", 0, 0, vec![b'1'], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Retrieve it
    let get = Get::one(GetInstr::Gets, "x");
    let cmd = Cmd::Get(get);
    let gets_resp = driver.run(cmd);
    // cas_unique is present
    gets_resp.get_first_value().unwrap().cas_unique.unwrap();

    // Set the key again
    let set = Set::new(SetInstr::Set, "x", 0, 0, vec![b'2'], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Retrieve it again - cas_unique should have changed
    let get = Get::one(GetInstr::Gets, "x");
    let cmd = Cmd::Get(get);
    let gets_resp2 = driver.run(cmd);
    // cas_unique has changed
    assert!(gets_resp.get_first_value().unwrap().cas_unique.unwrap() !=
            gets_resp2.get_first_value().unwrap().cas_unique.unwrap());
}


// Incr

#[test]
fn test_cmd_incr() {
    let cache = Cache::new(4096);
    let mut driver = Driver::new(cache);

    // Try to incr an invalid key
    let inc = Inc::new(IncInstr::Incr, "x", 4, false);
    let cmd = Cmd::Inc(inc);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::NotFound);

    // Try to incr an invalid key - noreply
    let inc = Inc::new(IncInstr::Incr, "x", 4, true);
    let cmd = Cmd::Inc(inc);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Set a key we can incr
    let set = Set::new(SetInstr::Set, "x", 0, 0, vec![b'1'], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Incr it
    let inc = Inc::new(IncInstr::Incr, "x", 1, false);
    let cmd = Cmd::Inc(inc);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::IntValue(2));

    // Make sure it was updated
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![b'2'], resp.get_first_value().unwrap().data);

    // Incr it again - noreply
    let inc = Inc::new(IncInstr::Incr, "x", 1, true);
    let cmd = Cmd::Inc(inc);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Make sure it was updated
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![b'3'], resp.get_first_value().unwrap().data);

    // Overflow it
    let inc = Inc::new(IncInstr::Incr, "x", 0xffffffffffffffff, false);
    let cmd = Cmd::Inc(inc);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::IntValue(2));

    // Set a key we can't incr - would not fit in u64
    let set = Set::new(SetInstr::Set, "y", 0, 0, vec![b'1'; 255], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Try to incr it - fails
    let inc = Inc::new(IncInstr::Incr, "y", 1, false);
    let cmd = Cmd::Inc(inc);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::ClientError("Not a number".to_string()));
}


// Prepend

#[test]
fn test_cmd_prepend() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    // Try to prepend to an invalid key
    let set = Set::new(SetInstr::Prepend, "x", 4, 0, vec![8, 9], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::NotStored);

    // Try to prepend to an invalid key - noreply
    let set = Set::new(SetInstr::Prepend, "x", 4, 0, vec![8, 9], true);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Set a key we can prepend to
    let set = Set::new(SetInstr::Set, "x", 0, 0, vec![8, 9], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Prepend to it
    let set = Set::new(SetInstr::Prepend, "x", 4, 0, vec![10], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // Make sure it was updated
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![10, 8, 9], resp.get_first_value().unwrap().data);
    assert_eq!(4, resp.get_first_value().unwrap().flags);

    // Prepend again, in noreply mode
    let set = Set::new(SetInstr::Prepend, "x", 5, 0, vec![11], true);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Make sure it was updated again
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![11, 10, 8, 9], resp.get_first_value().unwrap().data);
    assert_eq!(5, resp.get_first_value().unwrap().flags);
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

    // Try to replace an non-existent key - noreply
    let set = Set::new(SetInstr::Replace, "x", 0, 0, vec![8, 9], true);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

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
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
    let resp = driver.run(cmd);
    assert_eq!(vec![10], resp.get_first_value().unwrap().data);
    assert_eq!(4, resp.get_first_value().unwrap().flags);

    // Replace a valid key in noreply mode
    let set = Set::new(SetInstr::Replace, "x", 6, 0, vec![11], true);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Make sure it was updated
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
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
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
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
    let st_delete_misses = Stat::new("delete_misses", "1".to_string());
    let st_incr_hits = Stat::new("incr_hits", "0".to_string());
    let st_incr_misses = Stat::new("incr_misses", "0".to_string());
    let st_decr_hits = Stat::new("decr_hits", "0".to_string());
    let st_decr_misses = Stat::new("decr_misses", "0".to_string());
    let st_cas_hits = Stat::new("cas_hits", "0".to_string());
    let st_cas_misses = Stat::new("cas_misses", "0".to_string());
    let st_cas_badval = Stat::new("cas_badval", "0".to_string());
    let st_touch_hits = Stat::new("touch_hits", "0".to_string());
    let st_touch_misses = Stat::new("touch_misses", "0".to_string());
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
                     st_incr_hits,
                     st_incr_misses,
                     st_decr_hits,
                     st_decr_misses,
                     st_cas_hits,
                     st_cas_misses,
                     st_cas_badval,
                     st_touch_hits,
                     st_touch_misses,
                     st_bytes_read,
                     st_bytes_written,
                     st_limit_maxbytes,
                     st_bytes,
                     st_curr_items,
                     st_total_items,
                     st_evictions,
                     st_reclaimed]));
}


// Touch

// this is a slow test that relies on sleeps
#[ignore]
#[test]
fn test_cmd_touch() {
    let cache = Cache::new(100);
    let mut driver = Driver::new(cache);

    // Try to touch an invalid key
    let touch = Touch::new("x", 0, false);
    let cmd = Cmd::Touch(touch);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::NotFound);

    // Try to touch an invalid key - noreply
    let touch = Touch::new("x", 0, true);
    let cmd = Cmd::Touch(touch);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // Set a key that expires in 3s
    let set = Set::new(SetInstr::Set, "x", 0, 3, vec![8, 9], false);
    let cmd = Cmd::Set(set);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Stored);

    // sleep 1.5 secs - not long enough to expire key
    sleep_secs(1.5);

    // Touch the key to keep it alive (set same exptime)
    let touch = Touch::new("x", 3, false);
    let cmd = Cmd::Touch(touch);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Touched);

    // Touch it again - noreply
    let touch = Touch::new("x", 3, true);
    let cmd = Cmd::Touch(touch);
    let resp = driver.run(cmd);
    assert_eq!(resp, Resp::Empty);

    // sleep 1.5 secs - the key would have expired by now without being touched
    sleep_secs(1.5);

    // It's still there
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
    let resp = driver.run(cmd);
    assert_eq!(1, resp.get_values().unwrap().len());

    // sleep 2.5 secs - long enough to expire after the touch
    sleep_secs(2.5);

    // It's gone
    let cmd = Cmd::Get(Get::one(GetInstr::Get, "x"));
    let resp = driver.run(cmd);
    assert_eq!(0, resp.get_values().unwrap().len());
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
    let cmd = Cmd::Get(Get::one(GetInstr::Get, key_name));
    let resp = driver.run(cmd);
    assert_eq!(blob, resp.get_first_value().unwrap().data);

    // sleep 1.5 secs - long enough to expire key
    sleep_secs(1.5);

    // Retrieve the key again - it's gone
    let cmd = Cmd::Get(Get::one(GetInstr::Get, key_name));
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
    let cmd = Cmd::Get(Get::one(GetInstr::Get, key_name));
    let resp = driver.run(cmd);
    assert_eq!(blob, resp.get_first_value().unwrap().data);

    // sleep 2.5 secs - long enough to expire key
    sleep_secs(2.5);

    // Retrieve the key again - it's gone
    let cmd = Cmd::Get(Get::one(GetInstr::Get, key_name));
    let resp = driver.run(cmd);
    assert_eq!(0, resp.get_values().unwrap().len());
}
