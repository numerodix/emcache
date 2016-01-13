use rand::Rng;
use rand::thread_rng;


pub fn get_rand_f64_vec(lower: u64, upper: u64) -> Vec<f64> {
    // create the floats
    let mut items: Vec<f64> = (lower..upper + 1)
                                  .map(|x: u64| x as f64)
                                  .collect();

    // now shuffle them
    thread_rng().shuffle(&mut items);

    items
}


#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::get_rand_f64_vec;


    #[test]
    fn test_get_rand_f64_vec() {
        // construct a sorted version of the same vector
        let expected: Vec<f64> = (1..101).map(|x: u64| x as f64).collect();

        // get some random vectors
        let mut vec = get_rand_f64_vec(1, 100);
        let vec2 = get_rand_f64_vec(1, 100);

        // it isn't constant
        assert!(vec != vec2);

        // it isn't sorted
        assert!(vec != expected);

        // it has the right size
        assert_eq!(vec.len(), 100);

        // now sort the vector
        vec.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        // the sorted vector is now equal to the one we made ourselves
        assert_eq!(vec, expected);
    }
}
