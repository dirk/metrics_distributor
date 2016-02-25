pub mod datadog;

use super::metrics::AggregatedMetrics;

pub trait Forwarder {
    fn forward_metrics(&self, AggregatedMetrics);
}
