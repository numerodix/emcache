use std::collections::HashMap;
use std::sync::mpsc;

use platform::time::sleep_secs;
use platform::time::time_now;
use testlib::cmp::eq_f64;

use super::Duration;
use super::LiveTimers;
use super::MetricsRecorder;
use super::Second;
use super::StartTime;
use super::TimeSeries;
use super::Timer;


#[test]
#[should_panic]
fn test_live_timers_name_mismatch() {
    let mut lt = LiveTimers::new();

    lt.start("cmd");
    lt.stop("resp"); // panic expected
}

// this is a slow test that relies on sleeps
#[ignore]
#[test]
fn test_live_timers_ok() {
    let mut lt = LiveTimers::new();

    let t1 = time_now();
    let start_time = lt.start("cmd");
    // start_time is very close to *now*
    assert!(eq_f64(t1, start_time, 0.001));
    // "cmd" -> start_time was added to the map
    assert_eq!(&start_time, lt.get_timers().get("cmd").unwrap());

    sleep_secs(0.25);

    let (start_time2, duration) = lt.stop("cmd");
    // the returned start_time matches what we saw before
    assert_eq!(start_time, start_time2);
    // the duration is almost exactly the time we slept
    assert!(eq_f64(0.25, duration, 0.001));
    // "cmd" was removed from the map
    assert!(!lt.get_timers().contains_key("cmd"));
}


#[test]
fn test_time_series_updates() {
    let mut ts = TimeSeries::new();

    // add a timer
    ts.add_timer("cmd", 1.1, 0.25);
    // construct the expected value for comparison
    let expected = hashmap!{
        "cmd".to_string() => hashmap!{
            1 => vec![0.25],
        },
    };
    // compare
    assert_eq!(&expected, ts.get_timers());

    // add another timer
    ts.add_timer("cmd", 1.9, 0.51);
    // construct the expected value for comparison
    let expected = hashmap!{
        "cmd".to_string() => hashmap!{
            1 => vec![0.25, 0.51],
        },
    };
    // compare
    assert_eq!(&expected, ts.get_timers());

    // add another timer
    ts.add_timer("cmd", 2.3, 8.8);
    // construct the expected value for comparison
    let expected = hashmap!{
        "cmd".to_string() => hashmap!{
            1 => vec![0.25, 0.51],
            2 => vec![8.8],
        },
    };
    // compare
    assert_eq!(&expected, ts.get_timers());

    // add another timer
    ts.add_timer("resp", 4.1, 1.0);
    // construct the expected value for comparison
    let expected = hashmap!{
        "cmd".to_string() => hashmap!{
            1 => vec![0.25, 0.51],
            2 => vec![8.8],
        },
        "resp".to_string() => hashmap!{
            4 => vec![1.0],
        },
    };
    // compare
    assert_eq!(&expected, ts.get_timers());

    // empty the series
    ts.clear();
    // construct the expected value for comparison
    let expected = hashmap!{};
    // compare
    assert_eq!(&expected, ts.get_timers());
}

#[test]
fn test_time_series_merges() {
    let mut ts = TimeSeries::new();

    // add some timers to both series
    ts.add_timer("cmd1", 1.1, 0.21);
    let mut other = TimeSeries::new();
    other.add_timer("cmd1", 1.7, 98.3);
    // construct expected value
    let expected = hashmap!{
        "cmd1".to_string() => hashmap!{
            1 => vec![0.21, 98.3],
        },
    };
    // merge and compare
    ts.merge(&other);
    assert_eq!(&expected, ts.get_timers());

    // add a timer to the other series
    let mut other = TimeSeries::new();
    other.add_timer("cmd1", 6.4, 0.9);
    // construct expected value
    let expected = hashmap!{
        "cmd1".to_string() => hashmap!{
            1 => vec![0.21, 98.3],
            6 => vec![0.9],
        },
    };
    // merge and compare
    ts.merge(&other);
    assert_eq!(&expected, ts.get_timers());

    // add a timer to the other series
    let mut other = TimeSeries::new();
    other.add_timer("cmd2", 3.1, 9.1);
    // construct expected value
    let expected = hashmap!{
        "cmd1".to_string() => hashmap!{
            1 => vec![0.21, 98.3],
            6 => vec![0.9],
        },
        "cmd2".to_string() => hashmap!{
            3 => vec![9.1],
        },
    };
    // merge and compare
    ts.merge(&other);
    assert_eq!(&expected, ts.get_timers());
}


// this is a slow test that relies on sleeps
#[ignore]
#[test]
fn test_timer_correct() {
    let (met_tx, met_rx) = mpsc::channel();
    let mut rec = MetricsRecorder::new(met_tx);

    // use Timer to make one timing
    let t1 = time_now() as u64;
    let rv = {
        let _t = Timer::new(&mut rec, "cmd");
        sleep_secs(0.25);
        ()
    };

    // flush the metrics so we can see them
    rec.flush_metrics();

    // receive the metrics
    let metrics = met_rx.recv().unwrap();
    // verify that the timing is correct
    let dur = metrics.timers
                     .get_timers()
                     .get("cmd")
                     .unwrap()
                     .get(&t1)
                     .unwrap()[0];
    assert!(eq_f64(0.25, dur, 0.001));
}

// this is a slow test that relies on sleeps
#[ignore]
#[test]
fn test_timer_wrong_binding() {
    let (met_tx, met_rx) = mpsc::channel();
    let mut rec = MetricsRecorder::new(met_tx);

    // use Timer to make one timing
    let t1 = time_now() as u64;
    let rv = {
        // this binding discards the value right away!
        let _ = Timer::new(&mut rec, "cmd");
        sleep_secs(0.25);
        ()
    };

    // flush the metrics so we can see them
    rec.flush_metrics();

    // receive the metrics
    let metrics = met_rx.recv().unwrap();
    // verify that the timing is correct
    let dur = metrics.timers
                     .get_timers()
                     .get("cmd")
                     .unwrap()
                     .get(&t1)
                     .unwrap()[0];
    assert!(eq_f64(0.0, dur, 0.01));
}

// this is a slow test that relies on sleeps
#[ignore]
#[test]
fn test_timer_no_binding() {
    let (met_tx, met_rx) = mpsc::channel();
    let mut rec = MetricsRecorder::new(met_tx);

    // use Timer to make one timing
    let t1 = time_now() as u64;
    let rv = {
        // no binding means Timer does not live past the first line
        Timer::new(&mut rec, "cmd");
        sleep_secs(0.25);
        ()
    };

    // flush the metrics so we can see them
    rec.flush_metrics();

    // receive the metrics
    let metrics = met_rx.recv().unwrap();
    // verify that the timing is correct
    let dur = metrics.timers
                     .get_timers()
                     .get("cmd")
                     .unwrap()
                     .get(&t1)
                     .unwrap()[0];
    assert!(eq_f64(0.0, dur, 0.01));
}
