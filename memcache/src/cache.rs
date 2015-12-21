use std::collections::HashMap;


#[derive(Debug, PartialEq)]
pub enum CacheError {
    CapacityExceeded,
    KeyNotFound,
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


pub struct Cache {
    capacity: u64,
//    key_maxlen: u64,
//    value_maxlen: u64,
    storage: HashMap<Key, Value>,
}

impl Cache {
    fn new(capacity: u64) -> Cache {
        Cache {
            capacity: capacity,
            storage: HashMap::new(),
        }
    }

    fn get(&self, key: Key) -> CacheResult<&Value> {
        // check key size

        match self.storage.get(&key) {
            Some(value) => Ok(value),
            None => Err(CacheError::KeyNotFound),
        }
    }

    fn set(&mut self, key: Key, value: Value) -> CacheResult<()> {
        // check key & value sizes

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
        let mut cache = Cache::new(1);

        let key = Key::new(vec![1, 2, 3]);
        let value = Value::new(vec![4, 5, 6]);

        cache.set(key.clone(), value.clone());
        let value_found = cache.get(key).unwrap();

        assert_eq!(&value, value_found);
    }

    #[test]
    fn test_key_not_found() {
        let mut cache = Cache::new(1);

        // Retrieve a different key to the one set
        cache.set(Key::new(vec![1]), Value::new(vec![9]));
        let rv = cache.get(Key::new(vec![2]));

        assert_rv_eq(rv, CacheError::KeyNotFound);
    }

    #[test]
    fn test_store_beyond_capacity() {
        let mut cache = Cache::new(1);

        // we reached capacity
        let rv = cache.set(Key::new(vec![1]), Value::new(vec![9]));
        assert!(rv.is_ok());

        // overwrite is ok
        let rv = cache.set(Key::new(vec![1]), Value::new(vec![9]));
        assert!(rv.is_ok());

        // cannot store a new key
        let rv = cache.set(Key::new(vec![2]), Value::new(vec![9]));
        assert_rv_eq(rv, CacheError::CapacityExceeded);
    }

}
