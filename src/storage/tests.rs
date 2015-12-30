use platform::time::sleep_secs;
use platform::time::time_now;

use super::Cache;
use super::CacheError;
use super::Key;
use super::Value;


#[test]
fn test_set_one_key() {
    let mut cache = Cache::new(1024);

    let key = key!(1, 2, 3);
    let value = value!(4, 5, 6);

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
    let value_found = cache.get(&key).unwrap();
    assert_eq!(value, *value_found);
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
    let mut cache = Cache::new(3);

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
    let mut cache = Cache::new(4);

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

    // get: use a key that is too long
    {
        let rv = cache.get(&key!(1, 2));
        assert_eq!(rv.unwrap_err(), CacheError::KeyTooLong);
    }

    // contains_key: use a key that is too long
    {
        let rv = cache.contains_key(&key!(1, 2));
        assert_eq!(rv.unwrap_err(), CacheError::KeyTooLong);
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

#[test]
fn test_metrics() {
    // NOTE: The most crucial metric is bytes, so make sure to test every data
    // path that affects it.

    let mut cache = Cache::new(4);
    assert_eq!(cache.get_stats().bytes, 0);
    assert_eq!(cache.get_stats().total_items, 0);

    // Set a key
    cache.set(key!(1), value!(2, 3)).unwrap();
    assert_eq!(cache.get_stats().bytes, 3);
    assert_eq!(cache.get_stats().evictions, 0);
    assert_eq!(cache.get_stats().get_hits, 0);
    assert_eq!(cache.get_stats().get_misses, 0);
    assert_eq!(cache.get_stats().total_items, 1);

    // Set a different key, evicting the first
    cache.set(key!(5), value!(6, 7)).unwrap();
    assert_eq!(cache.get_stats().bytes, 3);
    assert_eq!(cache.get_stats().evictions, 1);
    assert_eq!(cache.get_stats().get_hits, 0);
    assert_eq!(cache.get_stats().get_misses, 0);
    assert_eq!(cache.get_stats().total_items, 2);

    // Re-set the key with a different value
    cache.set(key!(5), value!(6, 7, 8)).unwrap();
    assert_eq!(cache.get_stats().bytes, 4);
    assert_eq!(cache.get_stats().evictions, 1);
    assert_eq!(cache.get_stats().get_hits, 0);
    assert_eq!(cache.get_stats().get_misses, 0);
    assert_eq!(cache.get_stats().total_items, 3);

    // Retrieve the key successfully
    cache.get(&key!(5)).unwrap();
    assert_eq!(cache.get_stats().bytes, 4);
    assert_eq!(cache.get_stats().evictions, 1);
    assert_eq!(cache.get_stats().get_hits, 1);
    assert_eq!(cache.get_stats().get_misses, 0);
    assert_eq!(cache.get_stats().total_items, 3);

    // Test for the key successfully
    cache.contains_key(&key!(5)).unwrap();
    assert_eq!(cache.get_stats().bytes, 4);
    assert_eq!(cache.get_stats().evictions, 1);
    assert_eq!(cache.get_stats().get_hits, 2);
    assert_eq!(cache.get_stats().get_misses, 0);
    assert_eq!(cache.get_stats().total_items, 3);

    // Retrieve a key that doesn't exist
    cache.get(&key!(17)).unwrap_err();
    assert_eq!(cache.get_stats().bytes, 4);
    assert_eq!(cache.get_stats().evictions, 1);
    assert_eq!(cache.get_stats().get_hits, 2);
    assert_eq!(cache.get_stats().get_misses, 1);
    assert_eq!(cache.get_stats().total_items, 3);

    // Create an expired value
    let mut value = value!(10, 11, 12);
    value.set_exptime(time_now() - 1.0);

    // Set a key that expires immediately
    cache.set(key!(9), value).unwrap();
    assert_eq!(cache.get_stats().bytes, 4);
    assert_eq!(cache.get_stats().evictions, 2);
    assert_eq!(cache.get_stats().get_hits, 2);
    assert_eq!(cache.get_stats().get_misses, 1);
    assert_eq!(cache.get_stats().total_items, 4);

    // Retrieve expired key
    cache.get(&key!(9)).unwrap_err();
    assert_eq!(cache.get_stats().bytes, 0);
    assert_eq!(cache.get_stats().evictions, 2);
    assert_eq!(cache.get_stats().get_hits, 2);
    assert_eq!(cache.get_stats().get_misses, 2);
    assert_eq!(cache.get_stats().total_items, 4);
}
