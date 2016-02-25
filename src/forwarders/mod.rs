pub mod datadog;

pub use self::datadog::DatadogForwarder;

use super::metrics::AggregatedMetrics;

pub trait Forwarder {
    fn forward_metrics(&self, AggregatedMetrics);
}
