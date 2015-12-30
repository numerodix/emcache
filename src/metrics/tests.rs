use platform::time::sleep_secs;
use platform::time::time_now;
use testlib::cmp::eq_f64;

use super::LiveTimers;


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
