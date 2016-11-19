[![Build Status][travis-image]][travis-url]

# Metrics Distributor

Metrics distributor is a Rust library for streamlining the creation and operation of metrics aggregation services. You can use it to build a simple, multi-protocol collection and forwarding service for your metrics.

Distributors can **collect** over a number of protocols and formats:

- HTTP POST requests: [`LogDrainHandler`][]
  - Log drain (body is raw log lines)
    - Heroku dyno performance metrics: [`HerokuLogLineReader`][]
    - Standard metrics format: [`StandardLogLineReader`][]
  - Batch submission of arrays of metrics (currently WIP)
    - urlencoded form
    - JSON
    - Standard metrics format
- [StatsD protocol][]:
  - TCP connection: [`StatsdTcpListener`][]
  - UDP datagrams: [`StatsdUdpListener`][]

[StatsD protocol]: https://github.com/b/statsd_spec
[`LogDrainHandler`]: https://dirk.github.io/metrics_distributor/metrics_distributor/collectors/http/struct.LogDrainHandler.html
[`HerokuLogLineReader`]: https://dirk.github.io/metrics_distributor/metrics_distributor/parsers/log_line/struct.HerokuLogLineReader.html
[`StandardLogLineReader`]: https://dirk.github.io/metrics_distributor/metrics_distributor/parsers/log_line/struct.StandardLogLineReader.html
[`StatsdTcpListener`]: https://dirk.github.io/metrics_distributor/metrics_distributor/collectors/statsd/struct.StatsdTcpListener.html
[`StatsdUdpListener`]: https://dirk.github.io/metrics_distributor/metrics_distributor/collectors/statsd/struct.StatsdUdpListener.html

They can then **forward** aggregated metrics over a number of protocols:

- HTTP POST in simple metrics format
- Datadog API: [`DatadogForwarder`][]
- [Graphite] plaintext
- StatsD

[Graphite]: https://graphite.readthedocs.org/en/latest/feeding-carbon.html
[`DatadogForwarder`]: https://dirk.github.io/metrics_distributor/metrics_distributor/forwarders/datadog/struct.DatadogForwarder.html

## Building on macOS

The system OpenSSL on macOS is too outdated. To use the one installed by Homebrew, run the following commands before building:

```sh
export OPENSSL_INCLUDE_DIR=`brew --prefix openssl`/include
export OPENSSL_LIB_DIR=`brew --prefix openssl`/lib
export LDFLAGS=-L`brew --prefix openssl`/lib
```

### Configuration

Distributor uses code as configuration. Rather than parsing a configuration format, you configure your service by composing collectors and forwarders. This means you get the added advantage of the powerful Rust compiler checking your "configuration" for correctness and that it's very easy to customize or create entirely new collections/forwarders.

See the `examples/` folder for some common configurations:

- [`http_server.rs`][]: Simple log drain
- [`statsd_server.rs`][]: StatsD (UDP) server

[`http_server.rs`]: examples/http_server.rs
[`statsd_server.rs`]: examples/statsd_server.rs

It turns out that the number of lines needed to set up a few collectors and forwarders in code is almost exactly the same as you would need with YAML/TOML/etc.

## License

Licensed under the 3-clause BSD license. See [LICENSE](LICENSE) for details.

[travis-image]: https://travis-ci.org/dirk/metrics_distributor.svg
[travis-url]: https://travis-ci.org/dirk/metrics_distributor
