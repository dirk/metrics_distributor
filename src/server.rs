use iron::prelude::*;
use iron::middleware::Handler;
use iron::status::Status;
use std::io::Read;

use super::{SharedDistributor};
use super::metrics::Metric;
use super::reader::LogLineReader;

pub struct LogDrainHandler {
    distributor: SharedDistributor,
    readers: Vec<Box<LogLineReader>>,
}

impl LogDrainHandler {
    pub fn new(distributor: SharedDistributor, readers: Vec<Box<LogLineReader>>) -> LogDrainHandler {
        LogDrainHandler {
            distributor: distributor,
            readers: readers,
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

        self.distributor.record(metrics);

        Ok(Response::with((Status::Created)))
    }
}
