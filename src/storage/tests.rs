use platform::time::sleep_secs;
use platform::time::time_now;

use super::Cache;
use super::CacheError;
use super::Key;
use super::Value;


#[test]
fn test_cas_id() {
    let mut value = value!(1);
    assert_eq!(0, *value.get_cas_id());

    value.set_item(vec![2]);
    assert_eq!(1, *value.get_cas_id());

    value.set_flags(15);
    assert_eq!(2, *value.get_cas_id());

    value.set_exptime(0.0);
    assert_eq!(3, *value.get_cas_id());

    // Touch is never due to a client changing it, just us
    value.touch();
    assert_eq!(3, *value.get_cas_id());
}

#[test]
fn test_set_one_key() {
    let mut cache = Cache::new(1024);

    let key = key!(1, 2, 3);
    let mut value = value!(4, 5, 6);
    value.set_flags(15);

    // First set it
    let rv = cache.set(key.clone(), value.clone());
    assert!(rv.is_ok());

    // Then test for it
    let rv = cache.contains_key(&key);
    assert_eq!(rv.unwrap(), true);

    // Check the size of the cache
    assert_eq!(1, cache.len());

    // Test for a key that was not set
    let rv = cache.contains_key(&key!(9, 8));
    assert_eq!(rv.unwrap(), false);

    // Now fetch it
    {
        let value_found = cache.get(&key).unwrap();
        assert_eq!(value, *value_found);
    }

    // Now remove it
    let value_popped = cache.remove(&key).unwrap();
    assert_eq!(value, value_popped);

    // Now test for it
    let rv = cache.contains_key(&key);
    assert_eq!(rv.unwrap(), false);

    // Check the size of the cache
    assert_eq!(0, cache.len());
}

#[test]
fn test_key_not_found() {
    let mut cache = Cache::new(1024);

    // Set a key
    let rv = cache.set(key!(1), value!(9));
    assert!(rv.is_ok());

    // Retrieve a different key
    let rv = cache.get(&key!(2));
    assert_eq!(rv.unwrap_err(), CacheError::KeyNotFound);
}

#[test]
fn test_store_beyond_capacity_lru() {
    let item_size = key!(1).mem_size() as u64 + value!(1).mem_size() as u64;
    let mut cache = Cache::new(item_size);

    // we've now reached capacity
    let rv = cache.set(key!(1), value!(8));
    assert!(rv.is_ok());
    assert_eq!(cache.len(), 1);

    // write another key
    let rv = cache.set(key!(2), value!(9));
    assert!(rv.is_ok());
    assert_eq!(cache.len(), 1);

    // the first key is gone
    {
        let rv = cache.contains_key(&key!(1));
        assert_eq!(rv.unwrap(), false);
    }
    {
        let rv = cache.get(&key!(1));
        assert!(rv.is_err());
    }

    // the second key is present
    {
        let rv = cache.contains_key(&key!(2));
        assert_eq!(rv.unwrap(), true);
    }
    {
        let rv = cache.get(&key!(2));
        assert!(rv.is_ok());
    }

    // try to set an item that's bigger than the whole cache
    let rv = cache.set(key!(2, 3), value!(9, 10, 11));
    assert!(rv.is_err());
    assert_eq!(cache.len(), 1);

    // make sure the previous set attempt didn't evict anything
    let rv = cache.contains_key(&key!(2));
    assert_eq!(rv.unwrap(), true);

}

#[test]
fn test_multiple_evictions() {
    // Get a cache just big enough to store two items with short key/val
    let item_size = key!(1).mem_size() as u64 + value!(1).mem_size() as u64;
    let mut cache = Cache::new(item_size * 2);

    // Set a key
    let rv = cache.set(key!(1), value!(8));
    assert!(rv.is_ok());
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.get_stats().evictions, 0);

    // Set another key
    let rv = cache.set(key!(2), value!(9));
    assert!(rv.is_ok());
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.get_stats().evictions, 0);

    // Set an item so big it forces everything else to be evicted
    let rv = cache.set(key!(3), value!(9, 10, 11));
    assert!(rv.is_ok());
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.get_stats().evictions, 2);
}

#[test]
fn test_exceed_item_size_limits() {
    let mut cache = Cache::new(1024);
    cache.with_key_maxlen(1)
         .with_value_maxlen(1);

    // contains_key: use a key that is too long
    {
        let rv = cache.contains_key(&key!(1, 2));
        assert_eq!(rv.unwrap_err(), CacheError::KeyTooLong);
    }

    // get: use a key that is too long
    {
        let rv = cache.get(&key!(1, 2));
        assert_eq!(rv.unwrap_err(), CacheError::KeyTooLong);
    }

    // remove: use a key that is too long
    {
        let rv = cache.remove(&key!(1, 2));
        assert_eq!(rv.unwrap_err(), CacheError::KeyTooLong);
    }

    // set: use a key that is too long
    {
        let rv = cache.set(key!(1, 2), value!(9));
        assert_eq!(rv.unwrap_err(), CacheError::KeyTooLong);
    }

    // set: use a value that is too long
    {
        let rv = cache.set(key!(1), value!(9, 8));
        assert_eq!(rv.unwrap_err(), CacheError::ValueTooLong);
    }
}

#[test]
fn test_key_expired_lifetime() {
    // our cache has a lifetime of 0 secs - all keys are dead on store
    let mut cache = Cache::new(1024);
    cache.with_item_lifetime(0.0);

    let key = key!(1);
    let value = value!(9);

    // set a key
    let rv = cache.set(key.clone(), value);
    assert!(rv.is_ok());

    // try to retrieve it - it has expired
    let rv = cache.get(&key);
    assert_eq!(rv.unwrap_err(), CacheError::KeyNotFound);
}

#[test]
fn test_key_explicit_exptime() {
    // our cache has infinite lifetime
    let mut cache = Cache::new(1024);

    let key = key!(1);
    let mut value = value!(9);
    // set exptime in the past
    value.set_exptime(time_now() - 1.0);

    // set a key
    let rv = cache.set(key.clone(), value);
    assert!(rv.is_ok());

    // try to retrieve it - it has expired
    let rv = cache.get(&key);
    assert_eq!(rv.unwrap_err(), CacheError::KeyNotFound);
}

// this is a slow test that relies on sleeps
#[ignore]
#[test]
fn test_key_kept_alive_on_access() {
    // our cache has a lifetime of 2 secs
    let mut cache = Cache::new(1024);
    cache.with_item_lifetime(2.0);

    let key = key!(1);
    let value = value!(9);

    let rv = cache.set(key.clone(), value.clone());
    assert!(rv.is_ok());

    // sleep 1.5 secs - not long enough to expire key
    sleep_secs(1.5);

    // access key - it's there
    assert!(cache.get(&key).is_ok());

    // sleep 1 secs - not long enough to expire key
    sleep_secs(1.0);

    // access key - it's now been 2.5s since it was set, but it's been accessed
    // so we've kept it alive
    assert!(cache.get(&key).is_ok());

    // sleep 2.5 secs - long enough to expire key
    sleep_secs(2.5);

    // access key - it's gone
    assert!(cache.get(&key).is_err());
}

// this is a slow test that relies on sleeps
#[ignore]
#[test]
fn test_flush_all() {
    // our cache has a lifetime of 2 secs
    let mut cache = Cache::new(1024);
    cache.with_item_lifetime(2.0);

    // this item lives for 3s
    let key1 = key!(1);
    let mut value1 = value!(9);
    value1.set_exptime(time_now() + 3.0);
    let rv = cache.set(key1.clone(), value1.clone());
    assert!(rv.is_ok());

    // this item lives until cache lifetime
    let key2 = key!(2);
    let value2 = value!(8);
    let rv = cache.set(key2.clone(), value2.clone());
    assert!(rv.is_ok());

    // make all items dead in one second
    cache.flush_all(time_now() + 1.0).unwrap();

    // sleep until flush time kicks in
    sleep_secs(1.5);

    // access both keys - both have expired
    assert!(cache.get(&key1).is_err());
    assert!(cache.get(&key2).is_err());

    // set a new item that came after flush_all
    let key3 = key!(3);
    let value3 = value!(7);
    let rv = cache.set(key3.clone(), value3.clone());
    assert!(rv.is_ok());

    // it was not expired
    assert!(cache.get(&key3).is_ok());
}

#[test]
fn test_metrics() {
    // NOTE: The most crucial metric is bytes, so make sure to test every data
    // path that affects it.

    let item_size = key!(1).mem_size() as u64 + value!(1, 2).mem_size() as u64;
    let mut cache = Cache::new(item_size);
    assert_eq!(cache.get_stats().bytes, 0);
    assert_eq!(cache.get_stats().total_items, 0);

    // Set a key
    cache.set(key!(1), value!(2, 3)).unwrap();
    assert_eq!(cache.get_stats().bytes, item_size);
    assert_eq!(cache.get_stats().evictions, 0);
    assert_eq!(cache.get_stats().get_hits, 0);
    assert_eq!(cache.get_stats().get_misses, 0);
    assert_eq!(cache.get_stats().delete_hits, 0);
    assert_eq!(cache.get_stats().delete_misses, 0);
    assert_eq!(cache.get_stats().total_items, 1);

    // Set a different key, evicting the first
    cache.set(key!(5), value!(6, 7)).unwrap();
    assert_eq!(cache.get_stats().bytes, item_size);
    assert_eq!(cache.get_stats().evictions, 1);
    assert_eq!(cache.get_stats().get_hits, 0);
    assert_eq!(cache.get_stats().get_misses, 0);
    assert_eq!(cache.get_stats().delete_hits, 0);
    assert_eq!(cache.get_stats().delete_misses, 0);
    assert_eq!(cache.get_stats().total_items, 2);

    // Re-set the key with a different value
    cache.set(key!(5), value!(6, 8)).unwrap();
    assert_eq!(cache.get_stats().bytes, item_size);
    assert_eq!(cache.get_stats().evictions, 1);
    assert_eq!(cache.get_stats().get_hits, 0);
    assert_eq!(cache.get_stats().get_misses, 0);
    assert_eq!(cache.get_stats().delete_hits, 0);
    assert_eq!(cache.get_stats().delete_misses, 0);
    assert_eq!(cache.get_stats().total_items, 3);

    // Retrieve the key successfully
    cache.get(&key!(5)).unwrap();
    assert_eq!(cache.get_stats().bytes, item_size);
    assert_eq!(cache.get_stats().evictions, 1);
    assert_eq!(cache.get_stats().get_hits, 1);
    assert_eq!(cache.get_stats().get_misses, 0);
    assert_eq!(cache.get_stats().delete_hits, 0);
    assert_eq!(cache.get_stats().delete_misses, 0);
    assert_eq!(cache.get_stats().total_items, 3);

    // Test for the key successfully
    cache.contains_key(&key!(5)).unwrap();
    assert_eq!(cache.get_stats().bytes, item_size);
    assert_eq!(cache.get_stats().evictions, 1);
    assert_eq!(cache.get_stats().get_hits, 2);
    assert_eq!(cache.get_stats().get_misses, 0);
    assert_eq!(cache.get_stats().delete_hits, 0);
    assert_eq!(cache.get_stats().delete_misses, 0);
    assert_eq!(cache.get_stats().total_items, 3);

    // Retrieve a key that doesn't exist
    cache.get(&key!(17)).unwrap_err();
    assert_eq!(cache.get_stats().bytes, item_size);
    assert_eq!(cache.get_stats().evictions, 1);
    assert_eq!(cache.get_stats().get_hits, 2);
    assert_eq!(cache.get_stats().get_misses, 1);
    assert_eq!(cache.get_stats().delete_hits, 0);
    assert_eq!(cache.get_stats().delete_misses, 0);
    assert_eq!(cache.get_stats().total_items, 3);

    // Create an expired value
    let mut value = value!(11, 12);
    value.set_exptime(time_now() - 1.0);

    // Set a key that expires immediately
    cache.set(key!(9), value).unwrap();
    assert_eq!(cache.get_stats().bytes, item_size);
    assert_eq!(cache.get_stats().evictions, 2);
    assert_eq!(cache.get_stats().get_hits, 2);
    assert_eq!(cache.get_stats().get_misses, 1);
    assert_eq!(cache.get_stats().delete_hits, 0);
    assert_eq!(cache.get_stats().delete_misses, 0);
    assert_eq!(cache.get_stats().total_items, 4);

    // Retrieve expired key
    cache.get(&key!(9)).unwrap_err();
    assert_eq!(cache.get_stats().bytes, 0);
    assert_eq!(cache.get_stats().evictions, 2);
    assert_eq!(cache.get_stats().get_hits, 2);
    assert_eq!(cache.get_stats().get_misses, 2);
    assert_eq!(cache.get_stats().delete_hits, 0);
    assert_eq!(cache.get_stats().delete_misses, 0);
    assert_eq!(cache.get_stats().total_items, 4);

    // Set another key
    cache.set(key!(21), value!(12, 13)).unwrap();
    assert_eq!(cache.get_stats().bytes, item_size);
    assert_eq!(cache.get_stats().evictions, 2);
    assert_eq!(cache.get_stats().get_hits, 2);
    assert_eq!(cache.get_stats().get_misses, 2);
    assert_eq!(cache.get_stats().delete_hits, 0);
    assert_eq!(cache.get_stats().delete_misses, 0);
    assert_eq!(cache.get_stats().total_items, 5);

    // Delete it
    cache.remove(&key!(21)).unwrap();
    assert_eq!(cache.get_stats().bytes, 0);
    assert_eq!(cache.get_stats().evictions, 2);
    assert_eq!(cache.get_stats().get_hits, 2);
    assert_eq!(cache.get_stats().get_misses, 2);
    assert_eq!(cache.get_stats().delete_hits, 1);
    assert_eq!(cache.get_stats().delete_misses, 0);
    assert_eq!(cache.get_stats().total_items, 5);

    // Try to delete it again
    cache.remove(&key!(21)).unwrap_err();
    assert_eq!(cache.get_stats().bytes, 0);
    assert_eq!(cache.get_stats().evictions, 2);
    assert_eq!(cache.get_stats().get_hits, 2);
    assert_eq!(cache.get_stats().get_misses, 2);
    assert_eq!(cache.get_stats().delete_hits, 1);
    assert_eq!(cache.get_stats().delete_misses, 1);
    assert_eq!(cache.get_stats().total_items, 5);
}
