use std::collections::HashMap;

use platform::time::time_now;

use super::StartTime;
use super::Timing;


#[derive(Debug, Clone, PartialEq)]
pub struct LiveTimers {
    timers: HashMap<String, StartTime>,
}

impl LiveTimers {
    pub fn new() -> LiveTimers {
        LiveTimers { timers: HashMap::new() }
    }

    pub fn get_timers(&self) -> &HashMap<String, StartTime> {
        &self.timers
    }


    pub fn start(&mut self, name: &str) -> StartTime {
        let start_time = time_now();
        self.timers.insert(name.to_string(), start_time.clone());
        start_time
    }

    pub fn stop(&mut self, name: &str) -> Timing {
        let stop_time = time_now();

        let opt = self.timers.remove(name);
        if opt.is_none() {
            panic!("Tried to stop non-live timer: {:?}", name);
        }

        let start_time = opt.unwrap();
        let duration = stop_time - start_time;

        Timing::new(name, start_time, duration)
    }
}
