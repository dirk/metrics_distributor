use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, sleep, JoinHandle};
use std::time::Duration;

use metrics::*;

/// Internal storage of metrics data. Normally you will want a `SharedStore`
/// which wraps this in an `Arc<Mutex<BaseStore>>` for thread-safe sharing
/// and access.
pub struct BaseStore {
    counts: HashMap<Dimension, u64>,
    measures: HashMap<Dimension, Vec<f64>>,
    samples: HashMap<Dimension, f64>,
}

impl BaseStore {
    pub fn new() -> BaseStore {
        BaseStore {
            counts: HashMap::new(),
            measures: HashMap::new(),
            samples: HashMap::new(),
        }
    }

    pub fn record(&mut self, metrics: Vec<Metric>) {
        for metric in metrics {
            match metric {
                Count(name, value) => {
                    let count = self.counts.entry(Dimension::with_name(name)).or_insert(0);
                    *count += value;
                },
                Measure(name, value) => {
                    let values = self.measures.entry(Dimension::with_name(name)).or_insert(Vec::new());
                    values.push(value);
                },
                Sample(name, value) => {
                    let entry = self.samples.entry(Dimension::with_name(name)).or_insert(0.0);
                    *entry = value;
                }
            }
        }
    } // fn record

    pub fn flush(&mut self) -> AggregatedMetrics {
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

    /// Takes a `Vec` of metrics and stores them.
    pub fn record(&self, metrics: Vec<Metric>) {
        let mut store = self.shared.lock().unwrap();
        store.record(metrics)
    }

    /// Aggregates all the metrics currently in the store and returns an
    /// `AggregatedMetrics` with the aggregated values for those metrics.
    /// This will empty the store, so it will not have any metrics in it
    /// after calling this.
    pub fn flush(&self) -> AggregatedMetrics {
        let mut store = self.shared.lock().unwrap();
        store.flush()
    }

    /// Starts a thread that calls `flush` on itself at a certain rate. After
    /// flushing it calls the given callback with the aggregated metrics
    /// that were flushed.
    pub fn flush_every<F>(&self, interval: Duration, callback: F) -> Vec<JoinHandle<()>>
        where F: Fn(AggregatedMetrics) + Send + 'static {

        let shared = self.shared.clone();

        let (send, recv) = mpsc::channel();

        vec![
            // Aggregate and send onto the channel
            thread::spawn(move || {
                loop {
                    sleep(interval);

                    let aggregated = {
                        let mut store = shared.lock().unwrap();
                        store.flush()
                    };

                    send.send(aggregated).unwrap()
                }
            }),
            // Receive aggregated metrics and send them to the callback function
            thread::spawn(move || {
                for aggregated in recv {
                    callback(aggregated)
                }
            })
        ]
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
        expected_counts.insert(Dimension::with_name("foo"), 3);

        assert_eq!(store.counts, expected_counts)
    }

    #[test]
    fn it_records_measure() {
        let store = get_store_with_metrics();

        let mut expected_measures = HashMap::new();
        expected_measures.insert(Dimension::with_name("bar"), vec![3.4, 5.6]);

        assert_eq!(store.measures, expected_measures)
    }

    #[test]
    fn it_records_sample() {
        let store = get_store_with_metrics();

        let mut expected_samples = HashMap::new();
        expected_samples.insert(Dimension::with_name("baz"), 9.0);

        assert_eq!(store.samples, expected_samples)
    }
}
