//! Forwarders send aggregated metrics on to other services.

pub mod datadog;

pub use self::datadog::DatadogForwarder;

use super::metrics::AggregatedMetrics;

/// Handles forwarding on a set of aggregated metrics.
pub trait Forwarder {
    /// Sends a vector of aggregated metrics to the forwarder's destination.
    fn forward_metrics(&self, AggregatedMetrics);
}
