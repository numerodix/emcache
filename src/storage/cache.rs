use super::accounting_hash_map::AccountingHashMap;
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
    storage: AccountingHashMap,
}

impl Cache {
    pub fn new(capacity: u64) -> Cache {
        Cache {
            capacity: capacity,
            item_lifetime: -1.0,
            key_maxlen: 250, // 250b
            value_maxlen: 1048576, // 1mb
            storage: AccountingHashMap::new(),
        }
    }

    pub fn with_item_lifetime(&mut self, item_lifetime: f64) -> &mut Cache {
        self.item_lifetime = item_lifetime;
        self
    }

    pub fn with_key_maxlen(&mut self, key_maxlen: u64) -> &mut Cache {
        self.key_maxlen = key_maxlen;
        self
    }

    pub fn with_value_maxlen(&mut self, value_maxlen: u64) -> &mut Cache {
        self.value_maxlen = value_maxlen;
        self
    }


    fn check_key_len(&self, key: &Key) -> bool {
        key.len() as u64 <= self.key_maxlen
    }

    fn check_value_len(&self, value: &Value) -> bool {
        value.len() as u64 <= self.value_maxlen
    }

    fn value_is_alive(&self, value: &Value) -> bool {
        // if the value has an exptime set, that takes precedence
        if value.exptime > 0.0 {
            if value.exptime > time_now_utc() {
                return true;
            } else {
                return false;
            }
        }

        // if we have no lifetime setting then values are always live
        if self.item_lifetime < 0.0 {
            return true;
        }

        // otherwise use lifetime to determine liveness
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

        // Pop the value first
        let mut opt = self.storage.remove(key);

        // We didn't find it
        if opt.is_none() {
            return Err(CacheError::KeyNotFound);
        }

        // From here on we can assume we did find it
        // Now check if the value is still alive
        let mut value = opt.unwrap();
        if !self.value_is_alive(&value) {
            return Err(CacheError::KeyNotFound);
        }

        // Update the value to mark that it's been accessed just now
        value.touch();

        // Now we reinsert the key to refresh it
        self.storage.insert(key.clone(), value);

        // Load since we need to return it
        let value = self.storage.get(key).unwrap();

        // Return success
        Ok(value)
    }

    pub fn len(&self) -> usize {
        self.storage.len()
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
                // Remove the oldest item to make space
                self.storage.pop_back();
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
