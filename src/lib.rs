use std::collections::HashMap;

type Seconds = u8;

struct Metric {
    name: String,
    resolution: Seconds,
}

struct Count {
    metric: Metric,
    value: u64,
}
struct Measure {
    metric: Metric,
    values: Vec<f64>,
}
struct Sample {
    metric: Metric,
    value: f64,
}

struct Distributor {
    counts: HashMap<String, Count>,
    measures: HashMap<String, Measure>,
    sample: HashMap<String, Sample>,
}

trait LogLineReader {
    fn read(&str);
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
