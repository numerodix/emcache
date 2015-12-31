use std::collections::HashMap;

use super::Duration;
use super::Second;
use super::StartTime;
use super::Timing;
use super::statistics::AggregatedMetric;
use super::statistics::aggregate_metric;


#[derive(Debug, Clone, PartialEq)]
pub struct TimeSeries {
    // name -> { 1 -> [0.13, 0.41], 2 -> [0.42, 0.6] }
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

    pub fn aggregate_metrics(&self) -> HashMap<String, AggregatedMetric> {
        let mut agg_mets = HashMap::new();

        for (name, seconds) in self.timers.iter() {
            for (_, samples) in seconds.iter() {
                let agg = aggregate_metric(name, samples);
                agg_mets.insert(name.to_string(), agg);
            }
        }

        agg_mets
    }

    pub fn clear(&mut self) {
        self.timers.clear();
    }
}
