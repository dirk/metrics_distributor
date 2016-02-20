use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use metrics::*;

/// Internal storage of store data.
pub struct BaseStore {
    counts: HashMap<String, u64>,
    measures: HashMap<String, Vec<f64>>,

    // TODO: Implement samples
    // samples: HashMap<String, Sample>,

    /// For how long it will collect its metrics before reporting the
    /// accumulated value of the metric.
    flush_interval: Seconds,
}

impl BaseStore {
    pub fn new() -> BaseStore {
        BaseStore {
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

/// Thread-safe interface to the store. In most cases this is what you
/// want to use.
#[derive(Clone)]
pub struct SharedStore {
    shared: Arc<Mutex<BaseStore>>,
}

impl SharedStore {
    pub fn new() -> SharedStore {
        SharedStore {
            shared: Arc::new(Mutex::new(BaseStore::new())),
        }
    }

    pub fn record(&self, metrics: Vec<Metric>) {
        let mut Store = self.shared.lock().unwrap();

        Store.record(metrics)
    }
}
