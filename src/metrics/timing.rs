use super::Duration;
use super::StartTime;


#[derive(Debug, Clone, PartialEq)]
pub struct Timing {
    name: String,
    start_time: StartTime,
    duration: Duration,
}

impl Timing {
    pub fn new(name: &str,
               start_time: StartTime,
               duration: Duration)
               -> Timing {
        Timing {
            name: name.to_string(),
            start_time: start_time,
            duration: duration,
        }
    }
}
