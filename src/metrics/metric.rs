use super::Timing;


#[derive(Debug, Clone, PartialEq)]
pub enum Metric {
    Timing(Timing),
}
