use std;
use std::time::Duration;

use time;


pub fn time_now_utc() -> f64 {
    let ts = time::get_time();
    ts.sec as f64 + (ts.nsec as f64 / 1_000_000_000f64)
}

fn convert_secs_to_duration(duration: f64) -> Duration {
    // extract the seconds (before the decimal point)
    let secs: u64 = duration.floor() as u64;
    // obtain the rest (after the decimal point)
    let rest = duration - secs as f64;
    // convert the rest to nanoseconds
    let nanosecs: u32 = (1_000_000_000f64 * rest).round() as u32;

    Duration::new(secs, nanosecs)
}

pub fn sleep_secs(secs: f64) {
    let dur = convert_secs_to_duration(secs);
    std::thread::sleep(dur);
}


#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::convert_secs_to_duration;


    #[test]
    fn test_a_quarter_second() {
        let dur = convert_secs_to_duration(1.25f64);
        assert_eq!(dur, Duration::new(1u64, 250_000_000u32));
    }

    #[test]
    fn test_a_half_second() {
        let dur = convert_secs_to_duration(1.5f64);
        assert_eq!(dur, Duration::new(1u64, 500_000_000u32));
    }
}
