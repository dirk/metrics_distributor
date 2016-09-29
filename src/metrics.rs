use std::slice::Iter;
use std::cmp::Ordering::Equal;

pub type Seconds = u8;

pub use self::Metric::*;

/// Metric collected from a collector to be recorded in a store.
#[derive(Clone, Debug, PartialEq)]
pub enum Metric {
    Count(String, u64),
    Measure(String, f64),
    Sample(String, f64),
}

#[derive(Debug, PartialEq)]
pub enum AggregatedMetricType {
    Count,
    Measure,
    Sample,
}

/// The final value resulting from aggregating a metric's values.
pub type AggregatedMetric = (AggregatedMetricType, String, f64);

/// All the metrics in a given time interval coalesced into a single value for
/// each metric.
#[derive(Debug, PartialEq)]
pub struct AggregatedMetrics {
    metrics: Vec<AggregatedMetric>,
}

impl AggregatedMetrics {
    pub fn new() -> AggregatedMetrics {
        AggregatedMetrics {
            metrics: vec![],
        }
    }

    pub fn with_metrics(metrics: Vec<AggregatedMetric>) -> AggregatedMetrics {
        AggregatedMetrics {
            metrics: metrics,
        }
    }

    pub fn aggregate_counts<'a, I>(&mut self, counts: I)
        where I: Iterator<Item=(&'a str, &'a u64)>
    {
        for (name, value) in counts {
            self.metrics.push((AggregatedMetricType::Count, name.to_owned(), *value as f64))
        }
    }

    /// Rolls up all the given measures. The minimum, maximum, median,
    /// average (mean), and 95th percentile summary measures will all be
    /// emitted, as well as a total count of all the individual measures
    /// received in the period.
    pub fn aggregate_measures<'a, I>(&mut self, measures: I)
        where I: Iterator<Item=(&'a str, &'a Vec<f64>)>
    {
        use self::AggregatedMetricType::*;

        for (name, values) in measures {
            let mut sorted = values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Equal));

            let min          = *sorted.first().unwrap();
            let max          = *sorted.last().unwrap();
            let median       = sorted[sorted.len() / 2]; // TODO: Improve how we calculate the median
            let average      = sorted.iter().fold(0.0, |sum, val| { sum + val }) / (sorted.len() as f64);
            let percentile95 = sorted[(sorted.len() as f64 * 0.95) as usize];
            let percentile99 = sorted[(sorted.len() as f64 * 0.99) as usize];

            self.metrics.push((Measure, format!("{}.min",          name), min));
            self.metrics.push((Measure, format!("{}.max",          name), max));
            self.metrics.push((Measure, format!("{}.median",       name), median));
            self.metrics.push((Measure, format!("{}.avg",          name), average));
            self.metrics.push((Measure, format!("{}.95percentile", name), percentile95));
            self.metrics.push((Measure, format!("{}.99percentile", name), percentile99));

            self.metrics.push((Count,   format!("{}.count", name), sorted.len() as f64));
        }
    }

    pub fn aggregate_samples<'a, I>(&mut self, samples: I)
        where I: Iterator<Item=(&'a str, &'a f64)>
    {
        for (name, value) in samples {
            self.metrics.push((AggregatedMetricType::Sample, name.to_owned(), *value as f64))
        }
    }

    pub fn iter(&self) -> Iter<AggregatedMetric> {
        self.metrics.iter()
    }

    pub fn len(&self) -> usize {
        self.metrics.len()
    }
}
