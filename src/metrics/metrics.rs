use super::Metric;


#[derive(Debug, Clone, PartialEq)]
pub struct Metrics {
    pub metrics: Vec<Metric>,
}

impl Metrics {
    pub fn new() -> Metrics {
        Metrics { metrics: vec![] }
    }

    pub fn clear(&mut self) {
        self.metrics.clear();
    }

    pub fn first(&self) -> &Metric {
        &self.metrics[0]
    }

    pub fn push(&mut self, item: Metric) {
        self.metrics.push(item);
    }
}
