pub type Seconds = u8;

pub use self::Metric::*;

pub enum Metric {
    Count(String, u64),
    Measure(String, f64),
    Sample(String, f64),
}
