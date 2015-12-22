use std::collections::HashMap;


#[derive(Debug, PartialEq)]
pub enum CacheError {
    CapacityExceeded,
    KeyNotFound,
    KeyTooLong,
    ValueTooLong,
}

pub type CacheResult<T> = Result<T, CacheError>;


#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Key {
    item: Vec<u8>,
}

impl Key {
    fn new(item: Vec<u8>) -> Key {
        Key { item: item }
    }

    fn len(&self) -> usize {
        self.item.len()
    }
}

// key!(1, 2, 3) => Key { item: Vec<u8> = [1, 2, 3] }
macro_rules! key {
    ( $( $x:expr ),* ) => {
        {
            let mut vec = Vec::new();
            $(
                vec.push($x);
            )*
            Key::new(vec)
        }
    };
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Value {
    item: Vec<u8>,
}

impl Value {
    fn new(item: Vec<u8>) -> Value {
        Value { item: item }
    }

    fn len(&self) -> usize {
        self.item.len()
    }
}

// value!(1, 2, 3) => Value { item: Vec<u8> = [1, 2, 3] }
macro_rules! value {
    ( $( $x:expr ),* ) => {
        {
            let mut vec = Vec::new();
            $(
                vec.push($x);
            )*
            Value::new(vec)
        }
    };
}


pub struct Cache {
    capacity: u64,
    key_maxlen: u64,
    value_maxlen: u64,
    storage: HashMap<Key, Value>,
}

impl Cache {
    fn new(capacity: u64, key_maxlen: u64, value_maxlen: u64) -> Cache {
        Cache {
            capacity: capacity,
            key_maxlen: key_maxlen,
            value_maxlen: value_maxlen,
            storage: HashMap::new(),
        }
    }

    fn with_defaults(capacity: u64) -> Cache {
        Cache::new(
            capacity,
            1024, // key_maxlen = 1kb
            1048576 // value_maxlen = 1mb
            )
    }


    fn check_key_len(&self, key: &Key) -> bool {
        key.len() as u64 <= self.key_maxlen
    }

    fn check_value_len(&self, value: &Value) -> bool {
        value.len() as u64 <= self.value_maxlen
    }


    fn contains_key(&self, key: &Key) -> CacheResult<bool> {
        // Check key size
        if (!self.check_key_len(key)) {
            return Err(CacheError::KeyTooLong);
        }

        Ok(self.storage.contains_key(key))
    }

    fn get(&self, key: &Key) -> CacheResult<&Value> {
        // Check key size
        if (!self.check_key_len(key)) {
            return Err(CacheError::KeyTooLong);
        }

        match self.storage.get(key) {
            Some(value) => Ok(value),
            None => Err(CacheError::KeyNotFound),
        }
    }

    fn set(&mut self, key: Key, value: Value) -> CacheResult<()> {
        // Check key & value sizes
        if (!self.check_key_len(&key)) {
            return Err(CacheError::KeyTooLong);
        }
        if (!self.check_value_len(&value)) {
            return Err(CacheError::ValueTooLong);
        }

        // Check capacity if adding new key
        if (!self.storage.contains_key(&key)) {
            if (self.storage.len() as u64 == self.capacity) {
                return Err(CacheError::CapacityExceeded);
            }
        }

        self.storage.insert(key, value);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

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
        cache.set(key.clone(), value.clone());

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

        // Retrieve a different key to the one set
        cache.set(key!(1), value!(9));
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
