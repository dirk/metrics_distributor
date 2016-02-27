//! Collectors listen for metrics in various protocols. They record metrics
//! they receive in a `SharedStore`.

/// Tools for building collectors to be exposed through the Iron HTTP library.
pub mod http;

/// Provides UDP and TCP StatsD servers.
pub mod statsd;
