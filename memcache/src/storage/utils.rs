use std;
use time;


pub fn time_now_utc() -> i64 {
    let ts = time::get_time();
    ts.sec
}

pub fn sleep_secs(secs: u64) {
    let dur = std::time::Duration::new(secs, 0);
    std::thread::sleep(dur);
}
