use rand::Rng;
use rand::thread_rng;


pub fn get_rand_f64_vec(lower: u64, upper: u64) -> Vec<f64> {
    // create the floats
    let mut items: Vec<f64> = (lower..upper + 1)
                                  .map(|x: u64| x as f64)
                                  .collect();
    assert_eq!(upper - lower + 1, items.len() as u64);

    // now shuffle them
    thread_rng().shuffle(&mut items);

    items
}
