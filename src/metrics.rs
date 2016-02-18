pub type Seconds = u8;

pub struct Metric {
    pub name: String,
    pub resolution: Seconds,
}

impl Metric {
    pub fn new(name: String, resolution: Seconds) -> Metric {
        Metric {
            name: name,
            resolution: resolution,
        }
    }
}

pub struct Count {
    pub metric: Metric,
    pub value: u64,
}
pub struct Measure {
    pub metric: Metric,
    pub values: Vec<f64>,
}
pub struct Sample {
    pub metric: Metric,
    pub value: f64,
}
