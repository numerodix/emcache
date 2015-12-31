#[derive(Debug, Clone, PartialEq)]
pub struct AggregatedMetric {
    pub name: String,
    pub avg: Option<f64>,
    pub p0: Option<f64>,
    pub p90: Option<f64>,
    pub p99: Option<f64>,
    pub p999: Option<f64>,
}
