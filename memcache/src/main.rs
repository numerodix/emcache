mod storage;

use storage::Cache;
use storage::Key;
use storage::Value;


fn main() {
    let mut cache = Cache::with_defaults(1024);

    let key = Key::new(vec![1]);
    let value = Value::new(vec![9]);

    cache.set(key.clone(), value.clone()).unwrap();
    println!("Set key {:?} to value {:?}", key.item, value.item);

    let loaded = cache.get(&key).unwrap();
    println!("Loaded key {:?}, got value {:?}", key.item, loaded.item);
}
