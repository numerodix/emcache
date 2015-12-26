use std;
use std::time::Duration;

use time;


fn convert_secs_to_duration(duration: f64) -> Duration {
    // extract the seconds (before the decimal point)
    let secs: u64 = duration.floor() as u64;
    // obtain the rest (after the decimal point)
    let rest = duration - secs as f64;
    // convert the rest to nanoseconds
    let nanosecs: u32 = (1_000_000_000f64 * rest).round() as u32;

    Duration::new(secs, nanosecs)
}

fn convert_timespec_to_secs(ts: time::Timespec) -> f64 {
    ts.sec as f64 + (ts.nsec as f64 / 1_000_000_000f64)
}

pub fn time_now() -> f64 {
    let ts = time::get_time();
    convert_timespec_to_secs(ts)
}

pub fn sleep_secs(secs: f64) {
    let dur = convert_secs_to_duration(secs);
    std::thread::sleep(dur);
}


#[cfg(test)]
mod tests {
    use std::time::Duration;

    use time;

    use super::convert_secs_to_duration;
    use super::convert_timespec_to_secs;


    #[test]
    fn test_timespec_a_quarter_second() {
        let ts = time::Timespec::new(1i64, 250_000_000i32);
        assert_eq!(1.25f64, convert_timespec_to_secs(ts));
    }

    #[test]
    fn test_timespec_a_half_second() {
        let ts = time::Timespec::new(1i64, 500_000_000i32);
        assert_eq!(1.5f64, convert_timespec_to_secs(ts));
    }


    #[test]
    fn test_secs_a_quarter_second() {
        let dur = convert_secs_to_duration(1.25f64);
        assert_eq!(dur, Duration::new(1u64, 250_000_000u32));
    }

    #[test]
    fn test_secs_a_half_second() {
        let dur = convert_secs_to_duration(1.5f64);
        assert_eq!(dur, Duration::new(1u64, 500_000_000u32));
    }
}
