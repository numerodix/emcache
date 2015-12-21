use std::collections::HashMap;


#[derive(Debug)]
pub enum CacheError {
    CapacityExceeded,
    KeyNotFound,
}

pub type CacheResult<T> = Result<T, CacheError>;


pub struct Cache {
    capacity: u32,
//    key_maxlen: u32,
//    value_maxlen: u32,
    storage: HashMap<Vec<u8>, Vec<u8>>,
}


impl Cache {
    fn new(capacity: u32) -> Cache {
        Cache {
            capacity: capacity,
            storage: HashMap::new(),
        }
    }

    fn get(&self, key: Vec<u8>) -> CacheResult<&Vec<u8>> {
        // check key size

        match self.storage.get(&key) {
            Some(value) => Ok(value),
            None => Err(CacheError::KeyNotFound),
        }
    }

    fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> CacheResult<()> {
        // check key & value sizes

        // Check capacity if adding new key
        if (!self.storage.contains_key(&key)) {
            if (self.storage.len() as u32 == self.capacity) {
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

    #[test]
    fn test_set_one_key() {
        let mut cache = Cache::new(1);

        let key = [1, 2, 3];
        let value = [4, 5, 6];

        cache.set(key.clone(), value.clone());
        let value_found = cache.get(key).unwrap();

        assert_eq!(&value, value_found);
    }

    #[test]
    fn test_key_not_found() {
        let mut cache = Cache::new(1);

        // Retrieve a different key to the one set
        cache.set([1], [9]);
        let rv = cache.get([2]);

        assert_eq!(rv.unwrap(), CacheError::KeyNotFound);
    }

    #[test]
    fn test_store_beyond_capacity() {
        let mut cache = Cache::new(1);

        // we reached capacity
        let rv = cache.set([1], [9]);
        assert!(rv.is_ok());

        // overwrite is ok
        let rv = cache.set([1], [9]);
        assert!(rv.is_ok());

        // cannot store a new key
        let rv = cache.set([2], [9]);
        assert_eq!(rv.unwrap(), CacheError::CapacityExceeded);
    }
}
