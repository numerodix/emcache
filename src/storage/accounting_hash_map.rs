use linked_hash_map::LinkedHashMap;

use super::key::Key;
use super::value::Value;


// This is a pass through layer that simply forwards all operations to the
// underlying LinkedHashMap, all the while recording stats on operations that
// mutate the hashmap.
pub struct AccountingHashMap {
    storage: LinkedHashMap<Key, Value>,
}

impl AccountingHashMap {
    pub fn new() -> AccountingHashMap {
        AccountingHashMap {
            storage: LinkedHashMap::new(),
        }
    }


    pub fn contains_key(&self, key: &Key) -> bool {
        self.storage.contains_key(key)
    }

    pub fn get(&self, key: &Key) -> Option<&Value> {
        self.storage.get(key)
    }

    pub fn insert(&mut self, key: Key, value: Value) -> Option<Value> {
        self.storage.insert(key, value)
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn pop_back(&mut self) -> Option<(Key, Value)> {
        self.storage.pop_back()
    }

    pub fn remove(&mut self, key: &Key) -> Option<Value> {
        self.storage.remove(key)
    }
}
