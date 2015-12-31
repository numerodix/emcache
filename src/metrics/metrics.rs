use super::TimeSeries;


#[derive(Debug, Clone, PartialEq)]
pub struct Metrics {
    pub timers: TimeSeries,
}

impl Metrics {
    pub fn new() -> Metrics {
        Metrics { timers: TimeSeries::new() }
    }

    pub fn with_timers(&mut self, timers: TimeSeries) {
        self.timers = timers;
    }
}
