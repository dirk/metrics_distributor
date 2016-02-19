use std::collections::HashMap;

use metrics::*;

pub struct Distributor {
    counts: HashMap<String, Count>,
    measures: HashMap<String, Measure>,

    // TODO: Implement samples
    // samples: HashMap<String, Sample>,

    /// How often it will collect its metrics before reporting the accumulated
    /// value of the metric.
    default_resolution: Seconds,
}

impl Distributor {
    pub fn new() -> Distributor {
        Distributor {
            counts: HashMap::new(),
            measures: HashMap::new(),
            default_resolution: 10,
        }
    }

    pub fn record_measure(&mut self, name: String, value: f64) {
        let resolution = self.default_resolution;

        let measure = self.measures.entry(name.clone()).or_insert_with(|| {
            Measure {
                metric: Metric::new(name, resolution),
                values: vec![],
            }
        });

        measure.values.push(value)
    }

    pub fn record_count(&mut self, name: String, value: u64) {
        let resolution = self.default_resolution;

        let entry = self.counts.entry(name.clone()).or_insert_with(|| {
            Count {
                metric: Metric::new(name, resolution),
                value: 0,
            }
        });

        entry.value = value
    }
}
