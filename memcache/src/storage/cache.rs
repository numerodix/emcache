use std::collections::HashMap;

use super::errors::CacheError;
use super::key::Key;
use super::typedefs::CacheResult;
use super::value::Value;


pub struct Cache {
    capacity: u64,
    item_lifetime: i64,
    key_maxlen: u64,
    value_maxlen: u64,
    storage: HashMap<Key, Value>,
}

impl Cache {
    pub fn new(capacity: u64,
               item_lifetime: i64,
               key_maxlen: u64,
               value_maxlen: u64)
               -> Cache {
        Cache {
            capacity: capacity,
            item_lifetime: item_lifetime,
            key_maxlen: key_maxlen,
            value_maxlen: value_maxlen,
            storage: HashMap::new(),
        }
    }

    pub fn with_defaults(capacity: u64) -> Cache {
        Cache::new(capacity,
                   -1,  // item_lifetime = -1 (unlimited)
                   250, // key_maxlen = 250b
                   1048576 /* value_maxlen = 1mb */)
    }


    fn check_key_len(&self, key: &Key) -> bool {
        key.len() as u64 <= self.key_maxlen
    }

    fn check_value_len(&self, value: &Value) -> bool {
        value.len() as u64 <= self.value_maxlen
    }


    pub fn contains_key(&self, key: &Key) -> CacheResult<bool> {
        // Check key size
        if !self.check_key_len(key) {
            return Err(CacheError::KeyTooLong);
        }

        Ok(self.storage.contains_key(key))
    }

    pub fn get(&self, key: &Key) -> CacheResult<&Value> {
        // Check key size
        if !self.check_key_len(key) {
            return Err(CacheError::KeyTooLong);
        }

        match self.storage.get(key) {
            Some(value) => Ok(value),
            None => Err(CacheError::KeyNotFound),
        }
    }

    pub fn set(&mut self, key: Key, value: Value) -> CacheResult<()> {
        // Check key & value sizes
        if !self.check_key_len(&key) {
            return Err(CacheError::KeyTooLong);
        }
        if !self.check_value_len(&value) {
            return Err(CacheError::ValueTooLong);
        }

        // Check capacity if adding new key
        if !self.storage.contains_key(&key) {
            if self.storage.len() as u64 == self.capacity {
                return Err(CacheError::CapacityExceeded);
            }
        }

        self.storage.insert(key, value);
        Ok(())
    }
}
