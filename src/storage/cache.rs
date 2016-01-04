use linked_hash_map::LinkedHashMap;

use platform::time::time_now;

use super::errors::CacheError;
use super::key::Key;
use super::typedefs::CacheResult;
use super::value::Value;


struct CacheStats {
    pub bytes: u64, // Bytes currently stored
    pub evictions: u64, // Number of items removed to make space for new items
    pub get_hits: u64,
    pub get_misses: u64,
    pub delete_misses: u64,
    pub delete_hits: u64,
    pub reclaimed: u64, // Number of times an entry was reclaimed to store a new entry
    pub total_items: u64, // Total items stored since server started
}

impl CacheStats {
    pub fn new() -> CacheStats {
        CacheStats {
            bytes: 0,
            delete_hits: 0,
            delete_misses: 0,
            evictions: 0,
            get_hits: 0,
            get_misses: 0,
            reclaimed: 0,
            total_items: 0,
        }
    }

    fn bytes_add(&mut self, key: &Key, value: &Value) {
        self.bytes += key.len() as u64;
        self.bytes += value.len() as u64;
    }

    fn bytes_subtract(&mut self, key: &Key, value: &Value) {
        self.bytes -= key.len() as u64;
        self.bytes -= value.len() as u64;
    }
}


pub struct Cache {
    pub capacity: u64, // in bytes
    storage: LinkedHashMap<Key, Value>,
    item_lifetime: f64, // in seconds, <0 for unlimited
    global_exptime: f64, // unixtime, <0 for unset

    key_maxlen: u64, // in bytes
    value_maxlen: u64, // in bytes

    stats: CacheStats,
}

impl Cache {
    pub fn new(capacity: u64) -> Cache {
        Cache {
            capacity: capacity,
            global_exptime: -1.0,
            item_lifetime: -1.0,
            key_maxlen: 250, // 250b
            stats: CacheStats::new(),
            value_maxlen: 1048576, // 1mb
            storage: LinkedHashMap::new(),
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


    pub fn get_stats(&self) -> &CacheStats {
        &self.stats
    }


    fn check_key_len(&self, key: &Key) -> bool {
        key.len() as u64 <= self.key_maxlen
    }

    fn check_value_len(&self, value: &Value) -> bool {
        value.len() as u64 <= self.value_maxlen
    }


    fn evict_oldest(&mut self) -> CacheResult<(Key, Value)> {
        let opt = self.storage.pop_back();

        match opt {
            Some((key, value)) => {
                // Update stats
                self.stats.bytes_subtract(&key, &value);
                self.stats.evictions += 1;

                Ok((key, value))
            }
            None => Err(CacheError::EvictionFailed),
        }
    }

    fn value_is_alive(&self, value: &Value) -> bool {
        // If we have a global exptime set, then any item touched before it is
        // dead
        if self.global_exptime > 0.0 {
            if value.atime < self.global_exptime {
                return false;
            }
        }

        // If the value has an exptime set, that determines lifetime
        // regardless of item_lifetime in the cache
        if value.exptime > 0.0 {
            if self.global_exptime > 0.0 {
                if value.exptime < self.global_exptime {
                    return false;
                }
            }

            if value.exptime < time_now() {
                return false;
            } else {
                return true;
            }
        }

        // if we have no lifetime setting then values are always live
        if self.item_lifetime < 0.0 {
            return true;
        }

        // otherwise use lifetime to determine liveness
        value.atime + self.item_lifetime > time_now()
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

    pub fn flush_all(&mut self, exptime: f64) -> CacheResult<()> {
        self.global_exptime = exptime;
        Ok(())
    }

    pub fn get(&mut self, key: &Key) -> CacheResult<&Value> {
        // Check key size
        if !self.check_key_len(key) {
            return Err(CacheError::KeyTooLong);
        }

        // Pop the value first
        let opt = self.storage.remove(key);

        // We didn't find it
        if opt.is_none() {
            self.stats.get_misses += 1;
            return Err(CacheError::KeyNotFound);
        }

        // From here on we can assume we did find it
        let mut value = opt.unwrap();

        // The value has been successfully removed - update stats
        self.stats.bytes_subtract(key, &value);

        // Now check if the value is still alive
        if !self.value_is_alive(&value) {
            self.stats.get_misses += 1;
            return Err(CacheError::KeyNotFound);
        }

        // Update the value to mark that it's been accessed just now
        value.touch();

        // We are going to re-instate the key - update stats
        self.stats.bytes_add(key, &value);
        self.stats.get_hits += 1;

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

    pub fn remove(&mut self, key: &Key) -> CacheResult<Value> {
        // Check key size
        if !self.check_key_len(key) {
            return Err(CacheError::KeyTooLong);
        }

        let opt = self.storage.remove(key);

        match opt {
            Some(value) => {
                // Update stats
                self.stats.delete_hits += 1;
                self.stats.bytes_subtract(key, &value);

                Ok((value))
            }
            None => {
                // Update stats
                self.stats.delete_misses += 1;

                Err(CacheError::KeyNotFound)
            }
        }
    }

    pub fn set(&mut self, key: Key, mut value: Value) -> CacheResult<()> {
        // Check key & value sizes
        if !self.check_key_len(&key) {
            return Err(CacheError::KeyTooLong);
        }
        if !self.check_value_len(&value) {
            return Err(CacheError::ValueTooLong);
        }

        // Does this item even fit into our cache at all?
        if key.len() as u64 + value.len() as u64 > self.capacity {
            return Err(CacheError::CapacityExceeded);
        }

        // Do we already store this key?
        if self.storage.contains_key(&key) {
            let mut plus_delta = 0u64;
            {
                // Load the existing value
                let prev_value = self.storage.get(&key).unwrap();

                // We're updating the key, possibly with a different size value
                self.stats.bytes_subtract(&key, &prev_value);

                // Figure out how much more space we need to store the new value
                if value.len() > prev_value.len() {
                    plus_delta = value.len() as u64 - prev_value.len() as u64;
                }
            }

            // Would the new value exceed our capacity? Then we need to reclaim
            loop {
                if self.stats.bytes + key.len() as u64 + plus_delta <=
                   self.capacity {
                    break;
                }

                try!(self.evict_oldest());

                // Update stats
                self.stats.reclaimed += 1;
            }

        } else {
            // Do we have space for the new item?
            loop {
                if self.stats.bytes + key.len() as u64 +
                   value.len() as u64 <= self.capacity {
                    break;
                }

                try!(self.evict_oldest());

                // Update stats
                self.stats.reclaimed += 1;
            }
        }

        // Update stats
        self.stats.bytes_add(&key, &value);
        self.stats.total_items += 1;

        // Update atime for value
        value.touch();

        // Store the value
        self.storage.insert(key, value);

        // Return success
        Ok(())
    }
}
