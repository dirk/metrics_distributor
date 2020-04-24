use iron::prelude::*;
use iron::middleware::Handler;
use iron::status::Status;
use std::io::Read;

use super::super::SharedStore;
use super::super::metrics::Metric;
use super::super::parsers::log_line::LogLineReader;

/// Accepts HTTP requests and reads lines from the body. Each line will be
/// passed to its set of `readers` and any metrics collected by those readers
/// will be recorded in the `store`.
pub struct LogDrainHandler {
    store: SharedStore,
    readers: Vec<Box<dyn LogLineReader>>,
}

impl LogDrainHandler {
    /// Create a new handler with the given vector of readers. Each reader
    /// will be called on every line the handler receives.
    pub fn new(store: SharedStore, readers: Vec<Box<dyn LogLineReader>>) -> LogDrainHandler {
        LogDrainHandler {
            store,
            readers,
        }
    }
}

impl Handler for LogDrainHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let mut body = String::new();
        match req.body.read_to_string(&mut body) {
            Ok(_) => {},
            Err(error) => {
                println!("{:?}", error);
                return Err(IronError::new(error, Status::InternalServerError))
            },
        }

        let ref readers = self.readers;
        let mut metrics: Vec<Metric> = vec![];

        for line in body.lines() {
            for reader in readers {
                metrics.extend(reader.read(line))
            }
        }

        self.store.record(metrics);

        Ok(Response::with(Status::Created))
    }
}
