use super::Cache;
use super::CacheError;
use super::Key;
use super::Value;

use super::utils::sleep_secs;
use super::utils::time_now_utc;


#[test]
fn test_set_one_key() {
    let mut cache = Cache::new(1);

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
    let mut cache = Cache::new(1);

    // Set a key
    let rv = cache.set(key!(1), value!(9));
    assert!(rv.is_ok());

    // Retrieve a different key
    let rv = cache.get(&key!(2));
    assert_eq!(rv.unwrap_err(), CacheError::KeyNotFound);
}

#[test]
fn test_store_beyond_capacity_lru() {
    let mut cache = Cache::new(1);

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
        let rv = cache.get(&key!(1));
        assert!(rv.is_err());
    }

    // the second key is present
    {
        let rv = cache.get(&key!(2));
        assert!(rv.is_ok());
    }
}

#[test]
fn test_exceed_item_size_limits() {
    let mut cache = Cache::new(1);
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
    let mut cache = Cache::new(1);
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
    let mut cache = Cache::new(1);

    let key = key!(1);
    let mut value = value!(9);
    // set exptime in the past
    value.set_exptime(time_now_utc() - 1.0);

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
    let mut cache = Cache::new(1);
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
