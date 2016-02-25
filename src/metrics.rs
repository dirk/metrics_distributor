pub type Seconds = u8;

pub use self::Metric::*;

#[derive(Debug, PartialEq)]
pub enum Metric {
    Count(String, u64),
    Measure(String, f64),
    Sample(String, f64),
}
