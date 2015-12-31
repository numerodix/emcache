use std::collections::HashMap;

use super::Duration;
use super::Second;
use super::StartTime;
use super::Timing;
use super::statistics::AggregatedMetric;
use super::statistics::aggregate_metric;


#[derive(Debug, Clone, PartialEq)]
pub struct TimeSeries {
    timers: HashMap<String, HashMap<Second, Vec<Duration>>>,
}

impl TimeSeries {
    pub fn new() -> TimeSeries {
        TimeSeries { timers: HashMap::new() }
    }

    pub fn get_timers(&self) -> &HashMap<String, HashMap<Second, Vec<Duration>>> {
        &self.timers
    }


    pub fn add_timing(&mut self, timing: &Timing) {
        // does the name series exist?
        if !self.timers.contains_key(&timing.name) {
            self.timers.insert(timing.name.to_string(), HashMap::new());
        }

        // does the second series exist?
        let sec = timing.start_time as Second;
        if !self.timers.get(&timing.name).unwrap().contains_key(&sec) {
            self.timers.get_mut(&timing.name).unwrap().insert(sec, vec![]);
        }

        // insert the value
        self.timers
            .get_mut(&timing.name)
            .unwrap()
            .get_mut(&sec)
            .unwrap()
            .push(timing.duration);
    }

    pub fn clear(&mut self) {
        self.timers.clear();
    }

    pub fn compute_metrics(&self) -> HashMap<String, AggregatedMetric> {
        let mut comp_mets = HashMap::new();

        for (name, seconds) in self.timers.iter() {
            for (_, samples) in seconds.iter() {
                let comp = aggregate_metric(name, samples);
                comp_mets.insert(name.to_string(), comp);
            }
        }

        comp_mets
    }
}
