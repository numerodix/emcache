use std::collections::HashMap;


#[derive(Debug, Clone)]
pub struct Metrics {
    // name, duration
    timers: HashMap<String, f64>,
}

impl Metrics {
    pub fn new() -> Metrics {
        Metrics { timers: HashMap::new() }
    }

    pub fn with_timers(&mut self, timers: HashMap<String, f64>) {
        self.timers = timers;
    }
}
