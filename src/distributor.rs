use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use metrics::*;

/// Internal storage of distributor data.
pub struct BaseDistributor {
    counts: HashMap<String, u64>,
    measures: HashMap<String, Vec<f64>>,

    // TODO: Implement samples
    // samples: HashMap<String, Sample>,

    /// For how long it will collect its metrics before reporting the
    /// accumulated value of the metric.
    flush_interval: Seconds,
}

impl BaseDistributor {
    pub fn new() -> BaseDistributor {
        BaseDistributor {
            counts: HashMap::new(),
            measures: HashMap::new(),
            flush_interval: 10,
        }
    }

    fn record(&mut self, metrics: Vec<Metric>) {
        for metric in metrics {
            match metric {
                Count(name, value) => {
                    let count = self.counts.entry(name).or_insert(0);
                    *count += value;
                },
                Measure(name, value) => {
                    let values = self.measures.entry(name).or_insert(Vec::new());
                    values.push(value);
                },
            }
        }
    } // fn record
}

/// Thread-safe interface to the distributor. In most cases this is what you
/// want to use.
#[derive(Clone)]
pub struct SharedDistributor {
    shared: Arc<Mutex<BaseDistributor>>,
}

impl SharedDistributor {
    pub fn new() -> SharedDistributor {
        SharedDistributor {
            shared: Arc::new(Mutex::new(BaseDistributor::new())),
        }
    }

    pub fn record(&self, metrics: Vec<Metric>) {
        let mut distributor = self.shared.lock().unwrap();

        distributor.record(metrics)
    }
}
