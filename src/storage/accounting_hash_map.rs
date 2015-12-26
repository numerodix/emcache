use linked_hash_map::LinkedHashMap;

use super::key::Key;
use super::value::Value;


pub struct Stats {
    pub bytes: u64,
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            bytes: 0,
        }
    }
}


// This is a pass through layer that simply forwards all operations to the
// underlying LinkedHashMap, all the while recording stats on operations that
// mutate the hashmap.
pub struct AccountingHashMap {
    stats: Stats,
    storage: LinkedHashMap<Key, Value>,
}

impl AccountingHashMap {
    pub fn new() -> AccountingHashMap {
        AccountingHashMap {
            stats: Stats::new(),
            storage: LinkedHashMap::new(),
        }
    }


    pub fn get_stats(&self) -> &Stats {
        &self.stats
    }


    pub fn contains_key(&self, key: &Key) -> bool {
        self.storage.contains_key(key)
    }

    pub fn get(&self, key: &Key) -> Option<&Value> {
        self.storage.get(key)
    }

    pub fn insert(&mut self, key: Key, value: Value) -> Option<Value> {
        self.stats.bytes += (key.len() as u64 + value.len() as u64);

        self.storage.insert(key, value)
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn pop_back(&mut self) -> Option<(Key, Value)> {
        let opt = self.storage.pop_back();

        match opt {
            Some((ref key, ref value)) => {
                self.stats.bytes -= (key.len() as u64 + value.len() as u64);
            },
            None => (),
        }

        opt
    }

    pub fn remove(&mut self, key: &Key) -> Option<Value> {
        let opt = self.storage.remove(key);

        match opt {
            Some(ref value) => {
                self.stats.bytes -= (key.len() as u64 + value.len() as u64);
            },
            None => (),
        }

        opt
    }
}
