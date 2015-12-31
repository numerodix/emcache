use super::Timing;


#[derive(Debug, Clone, PartialEq)]
pub enum Metric {
    Timing(Timing),
}

impl Metric {
    pub fn get_timing(&self) -> &Timing {
        match self {
            &Metric::Timing(ref timing) => &timing,
        }
    }
}
