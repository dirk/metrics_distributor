use std::collections::hash_map;
use std::slice::Iter;
use std::cmp::Ordering::Equal;

pub type Seconds = u8;

pub use self::Metric::*;

#[derive(Debug, PartialEq)]
pub enum Metric {
    Count(String, u64),
    Measure(String, f64),
    Sample(String, f64),
}

pub type AggregatedMetric = (String, f64);

/// All the metrics in a given time interval coalesced into a single value for each metric.
pub struct AggregatedMetrics {
    metrics: Vec<AggregatedMetric>,
}

impl AggregatedMetrics {
    pub fn new() -> AggregatedMetrics {
        AggregatedMetrics {
            metrics: vec![],
        }
    }

    pub fn aggregate_counts(&mut self, counts: hash_map::Iter<String, u64>) {
        for (name, value) in counts {
            self.metrics.push((name.to_owned(), *value as f64))
        }
    }

    pub fn aggregate_measures(&mut self, measures: hash_map::Iter<String, Vec<f64>>) {
        for (name, values) in measures {
            let mut sorted = values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Equal));

            let min          = *sorted.first().unwrap();
            let max          = *sorted.last().unwrap();
            let median       = sorted[(sorted.len() / 2) - 1];
            let average      = sorted.iter().fold(0.0, |sum, val| { sum + val }) / (sorted.len() as f64);
            let percentile95 = sorted[(sorted.len() as f64 * 0.95) as usize];

            self.metrics.push((format!("{}.min",          name.clone()), min));
            self.metrics.push((format!("{}.max",          name.clone()), max));
            self.metrics.push((format!("{}.median",       name.clone()), median));
            self.metrics.push((format!("{}.avg",          name.clone()), average));
            self.metrics.push((format!("{}.95percentile", name.clone()), percentile95));

            self.metrics.push((format!("{}.count", name.clone()), sorted.len() as f64));
        }
    }

    pub fn aggregate_samples(&mut self, samples: hash_map::Iter<String, f64>) {
        for (name, value) in samples {
            self.metrics.push((name.to_owned(), *value as f64))
        }
    }

    pub fn iter(&self) -> Iter<AggregatedMetric> {
        self.metrics.iter()
    }
}
