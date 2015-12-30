use rand::Rng;
use rand::thread_rng;

use super::ComputedMetric;
use super::compute_average;
use super::compute_metric;
use super::compute_p90;
use super::compute_p999;
use super::compute_p99;
use super::sort_f64;


fn get_rand_f64_vec(lower: u64, upper: u64) -> Vec<f64> {
    // create the floats
    let mut items: Vec<f64> = (lower..upper + 1)
                                  .map(|x: u64| x as f64)
                                  .collect();
    assert_eq!(upper - lower + 1, items.len() as u64);

    // now shuffle them
    thread_rng().shuffle(&mut items);

    items
}


#[test]
fn test_get_rand_f64_vec() {
    let vals = get_rand_f64_vec(1, 100);
    assert_eq!(100, vals.len());
    assert_eq!(5050.0, vals.iter().fold(0.0, |acc, x| acc + x));
}


#[test]
fn test_compute_average_empty() {
    assert_eq!(None, compute_average(&vec![]));
}

#[test]
fn test_compute_average_ok() {
    assert_eq!(1.3, compute_average(&vec![1.1, 1.3, 1.5]).unwrap());
}


#[test]
fn test_sort_f64() {
    assert_eq!(&vec![1.2, 3.1, 9.1], sort_f64(&mut vec![9.1, 1.2, 3.1]));
}


#[test]
fn test_compute_p90_too_short() {
    let vals = get_rand_f64_vec(1, 9);
    assert_eq!(None, compute_p90(&vals));
}

#[test]
fn test_compute_p90_small() {
    let vals = get_rand_f64_vec(1, 10);
    assert_eq!(10.0, compute_p90(&vals).unwrap());
}

#[test]
fn test_compute_p90_large() {
    let vals = get_rand_f64_vec(1, 100);
    assert_eq!(91.0, compute_p90(&vals).unwrap());
}


#[test]
fn test_compute_p99_too_short() {
    let vals = get_rand_f64_vec(1, 99);
    assert_eq!(None, compute_p99(&vals));
}

#[test]
fn test_compute_p99_small() {
    let vals = get_rand_f64_vec(1, 100);
    assert_eq!(100.0, compute_p99(&vals).unwrap());
}

#[test]
fn test_compute_p99_large() {
    let vals = get_rand_f64_vec(1, 1000);
    assert_eq!(991.0, compute_p99(&vals).unwrap());
}


#[test]
fn test_compute_p999_too_short() {
    let vals = get_rand_f64_vec(1, 999);
    assert_eq!(None, compute_p999(&vals));
}

#[test]
fn test_compute_p999_small() {
    let vals = get_rand_f64_vec(1, 1000);
    assert_eq!(1000.0, compute_p999(&vals).unwrap());
}

#[test]
fn test_compute_p999_large() {
    let vals = get_rand_f64_vec(1, 10000);
    assert_eq!(9991.0, compute_p999(&vals).unwrap());
}


#[test]
fn test_compute_metric() {
    let vals = get_rand_f64_vec(1, 1000);
    let metric = compute_metric("latency", &vals);

    let expected = ComputedMetric {
        name: "latency".to_string(),
        avg: Some(500.5),
        p90: Some(901.0),
        p99: Some(991.0),
        p999: Some(1000.0),
    };
    assert_eq!(expected, metric);
}
