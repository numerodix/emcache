use std::collections::HashMap;

use platform::time::time_now;


pub struct MetricsRecorder {
    timers: HashMap<String, f64>,

    cur_timer: Option<String>,
    start_time: f64,
}

impl MetricsRecorder {
    pub fn new() -> MetricsRecorder {
        MetricsRecorder {
            timers: HashMap::new(),

            cur_timer: None,
            start_time: -1.0,
        }
    }

    pub fn start_timer(&mut self, name: &str) {
        self.cur_timer = Some(name.to_string());
        self.start_time = time_now();
    }

    pub fn stop_timer(&mut self, name: &str) {
        assert_eq!(self.cur_timer.as_ref().unwrap(), name);

        let duration = time_now() - self.start_time;
        self.start_time = -1.0;

        self.timers.insert(name.to_string(), duration);
        println!("Timed {:13}: {:?}s", name, duration);
    }
}

