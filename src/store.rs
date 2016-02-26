use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep, JoinHandle};
use std::time::Duration;

use metrics::*;

/// Internal storage of store data.
pub struct BaseStore {
    counts: HashMap<String, u64>,
    measures: HashMap<String, Vec<f64>>,
    samples: HashMap<String, f64>,
}

impl BaseStore {
    pub fn new() -> BaseStore {
        BaseStore {
            counts: HashMap::new(),
            measures: HashMap::new(),
            samples: HashMap::new(),
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
                Sample(name, value) => {
                    let entry = self.samples.entry(name).or_insert(0.0);
                    *entry = value;
                }
            }
        }
    } // fn record

    fn flush(&mut self) -> AggregatedMetrics {
        let mut aggregated = AggregatedMetrics::new();

        aggregated.aggregate_counts(self.counts.iter());
        self.counts.clear();

        aggregated.aggregate_measures(self.measures.iter());
        self.measures.clear();

        aggregated.aggregate_samples(self.samples.iter());
        self.samples.clear();

        aggregated
    } // fn flush
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
        let mut store = self.shared.lock().unwrap();
        store.record(metrics)
    }

    pub fn flush(&self) -> AggregatedMetrics {
        let mut store = self.shared.lock().unwrap();
        store.flush()
    }

    pub fn flush_every<F>(&self, interval: Duration, callback: F) -> JoinHandle<()>
        where F: Fn(AggregatedMetrics) + Send + 'static {

        let shared = self.shared.clone();

        thread::spawn(move || {
            loop {
                sleep(interval);

                let mut store = shared.lock().unwrap();
                let aggregated = store.flush();

                callback(aggregated);
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::BaseStore;
    use super::super::metrics::*;

    fn get_store_with_metrics() -> BaseStore {
        let metrics = vec![
            Count("foo".to_owned(), 1),
            Count("foo".to_owned(), 2),
            Measure("bar".to_owned(), 3.4),
            Measure("bar".to_owned(), 5.6),
            Sample("baz".to_owned(), 7.8),
            Sample("baz".to_owned(), 9.0),
        ];

        let mut store = BaseStore::new();

        store.record(metrics);

        store
    }

    #[test]
    fn it_records_count() {
        let store = get_store_with_metrics();

        let mut expected_counts = HashMap::new();
        expected_counts.insert("foo".to_owned(), 3);

        assert_eq!(store.counts, expected_counts)
    }

    #[test]
    fn it_records_measure() {
        let store = get_store_with_metrics();

        let mut expected_measures: HashMap<String, Vec<f64>> = HashMap::new();
        expected_measures.insert("bar".to_owned(), vec![3.4, 5.6]);

        assert_eq!(store.measures, expected_measures)
    }

    #[test]
    fn it_records_sample() {
        let store = get_store_with_metrics();

        let mut expected_samples = HashMap::new();
        expected_samples.insert("baz".to_owned(), 9.0);

        assert_eq!(store.samples, expected_samples)
    }
}
