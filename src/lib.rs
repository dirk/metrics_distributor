#[macro_use]
extern crate lazy_static;

extern crate regex;

mod reader;
mod metrics;

use std::collections::HashMap;
use metrics::*;

struct Distributor {
    counts: HashMap<String, Count>,
    measures: HashMap<String, Measure>,
    sample: HashMap<String, Sample>,
    default_resolution: Seconds,
}

impl Distributor {
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

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
