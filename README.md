# Metrics Distributor

**Note**: This documentation does not reflect the currently functionality. Instead it is an outline of functionality to be reached for initial release.

Metrics distributor is a Rust library for streamlining the creation and operation of metrics aggregation services. You can use it to build a simple, multi-protocol collection and forwarding service for your metrics.

Distributor can **collect** over a number of protocols and formats:

- HTTP POST requests
  - Log drain (body is raw log lines)
    - Heroku dyno performance metrics
    - Simple metrics format
  - Batch submission (array of metrics)
    - urlencoded form
    - JSON
    - Simple metrics format
- [StatsD protocol][] over TCP/UDP

[StatsD protocol]: https://github.com/b/statsd_spec

It can then **forward** aggregated metrics over a number of protocols:

- HTTP POST in simple metrics format
- Datadog API
- [Graphite] plaintext
- StatsD

[Graphite]: https://graphite.readthedocs.org/en/latest/feeding-carbon.html

### Configuration

Distributor uses code as configuration. Rather than parsing a configuration format, you configure your service by composing collectors and forwarders. This means you get the added advantage of the powerful Rust compiler checking your "configuration" for correctness.

This also means it's very easy to customize or create entirely new collections/forwarders.

It turns out that the number of lines needed to set up a few collectors and forwarders in code is almost exactly the same as you would need with YAML/TOML/etc.
