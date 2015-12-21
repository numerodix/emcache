use std::collections::HashMap;


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

    fn get(&self, key: Vec<u8>) -> Option<&Vec<u8>> {
        // check key size

        self.storage.get(&key)
    }

    fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        // check key & value sizes
        // check capacity

        self.storage.insert(key, value);
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
}
