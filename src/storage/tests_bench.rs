use test::Bencher;

use super::Cache;
use super::Key;
use super::Value;


#[bench]
fn bench_set_get_key(b: &mut Bencher) {
    let mut cache = Cache::new(1024);

    b.iter(|| {
        cache.set(key!(1), value!(9));
        cache.get(&key!(1));
    })
}
