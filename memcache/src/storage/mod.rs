// declare sub modules
pub mod cache;
pub mod errors;
pub mod key;
pub mod typedefs;
pub mod value;


#[cfg(test)]
mod tests {
    use super::cache::Cache;
    use super::errors::CacheError;
    use super::key::*;
    use super::typedefs::CacheResult;
    use super::value::*;


    // helper func since assert_eq!(rv.unwrap(), err) does not work
    fn assert_rv_eq<T>(rv: CacheResult<T>, err: CacheError) {
        // First of all it's supposed to be an error
        assert!(rv.is_err());

        // Now check if the error constructor is the right one
        match rv {
            Ok(_) => (),
            Err(e) => { assert_eq!(e, err); }
        };
    }


    #[test]
    fn test_set_one_key() {
        let mut cache = Cache::with_defaults(1);

        let key = key!(1, 2, 3);
        let value = value!(4, 5, 6);

        // First set it
        let rv = cache.set(key.clone(), value.clone());
        assert!(rv.is_ok());

        // Then test for it
        let rv = cache.contains_key(&key);
        assert_eq!(rv.unwrap(), true);

        // Test for a key that was not set
        let rv = cache.contains_key(&key!(9, 8));
        assert_eq!(rv.unwrap(), false);

        // Now fetch it
        let value_found = cache.get(&key).unwrap();

        assert_eq!(value, *value_found);
    }

    #[test]
    fn test_key_not_found() {
        let mut cache = Cache::with_defaults(1);

        // Set a key
        let rv = cache.set(key!(1), value!(9));
        assert!(rv.is_ok());

        // Retrieve a different key
        let rv = cache.get(&key!(2));
        assert_rv_eq(rv, CacheError::KeyNotFound);
    }

    #[test]
    fn test_store_beyond_capacity() {
        let mut cache = Cache::with_defaults(1);

        // we've now reached capacity
        let rv = cache.set(key!(1), value!(9));
        assert!(rv.is_ok());

        // overwriting is ok
        let rv = cache.set(key!(1), value!(9));
        assert!(rv.is_ok());

        // but we cannot store a new key
        let rv = cache.set(key!(2), value!(9));
        assert_rv_eq(rv, CacheError::CapacityExceeded);
    }

    #[test]
    fn test_exceed_item_size_limits() {
        let mut cache = Cache::new(1, 1, 1);

        // set: use a key that is too long
        let rv = cache.set(key!(1, 2), value!(9));
        assert_rv_eq(rv, CacheError::KeyTooLong);

        // set: use a value that is too long
        let rv = cache.set(key!(1), value!(9, 8));
        assert_rv_eq(rv, CacheError::ValueTooLong);

        // get: use a key that is too long
        let rv = cache.get(&key!(1, 2));
        assert_rv_eq(rv, CacheError::KeyTooLong);

        // contains_key: use a key that is too long
        let rv = cache.contains_key(&key!(1, 2));
        assert_rv_eq(rv, CacheError::KeyTooLong);
    }
}
