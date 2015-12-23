use std::collections::HashMap;

use super::errors::CacheError;
use super::key::Key;
use super::typedefs::CacheResult;
use super::utils::time_now_utc;
use super::value::Value;


pub struct Cache {
    capacity: u64,
    item_lifetime: f64, // in seconds, <0 for unlimited
    key_maxlen: u64, // in bytes
    value_maxlen: u64, // in bytes
    storage: HashMap<Key, Value>,
}

impl Cache {
    pub fn new(capacity: u64,
               item_lifetime: f64,
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
                   -1.0, // item_lifetime = -1.0
                   250, // key_maxlen = 250b
                   1048576 /* value_maxlen = 1mb */)
    }


    fn check_key_len(&self, key: &Key) -> bool {
        key.len() as u64 <= self.key_maxlen
    }

    fn check_value_len(&self, value: &Value) -> bool {
        value.len() as u64 <= self.value_maxlen
    }

    fn value_is_alive(&self, value: &Value) -> bool {
        if self.item_lifetime < 0.0 {
            return true;
        }

        value.atime + self.item_lifetime > time_now_utc()
    }

    fn remove(&mut self, key: &Key) -> CacheResult<()> {
        let opt = self.storage.remove(key);

        match opt {
            Some(_) => Ok(()),
            None => Err(CacheError::KeyNotFound),
        }
    }


    pub fn contains_key(&mut self, key: &Key) -> CacheResult<bool> {
        let result = self.get(key);

        match result {
            // We know how to interpret found and not found
            Ok(_) => Ok(true),
            Err(CacheError::KeyNotFound) => Ok(false),

            // Some other error
            Err(x) => Err(x),
        }
    }

    pub fn get(&mut self, key: &Key) -> CacheResult<&Value> {
        // Check key size
        if !self.check_key_len(key) {
            return Err(CacheError::KeyTooLong);
        }

        let mut is_alive = false;
        {
            // Retrieve the key
            let opt = self.storage.get(key);

            // We didn't find it
            if opt.is_none() {
                return Err(CacheError::KeyNotFound);
            }

            // From here on we can assume we did find it
            let value: &Value = opt.unwrap();
            if self.value_is_alive(value) {
                is_alive = true;
            }
        }

        // If the key is dead we evict it and return an error
        if !is_alive {
            self.remove(key).unwrap();
            return Err(CacheError::KeyNotFound);
        }

        // Otherwise we retrieve the key again, this time mutable
        let value = self.storage.get_mut(key).unwrap();

        // Update the value to mark that it's been accessed just now
        value.touch();

        // Return success
        Ok(value)
    }

    pub fn set(&mut self, key: Key, mut value: Value) -> CacheResult<()> {
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

        // Update atime for value
        value.touch();

        // Store the value
        self.storage.insert(key, value);

        // Return success
        Ok(())
    }
}
