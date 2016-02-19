use iron::prelude::*;

use super::Distributor;
use super::reader::LogLineReader;

struct LogDrainHandler<'a> {
    distributor: &'a mut Distributor,
    readers: Vec<Box<LogLineReader>>,
}
