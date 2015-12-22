use time;


pub fn time_now_utc() -> i64 {
    let ts = time::get_time();
    ts.sec
}
