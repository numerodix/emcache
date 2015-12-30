#[derive(Debug, Clone, PartialEq)]
pub struct ComputedMetric {
    pub name: String,
    pub avg: Option<f64>,
    pub p90: Option<f64>,
    pub p99: Option<f64>,
    pub p999: Option<f64>,
}
