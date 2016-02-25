use regex::{Regex};
use std::str::{FromStr};

use super::metrics::*;

pub trait LogLineReader: Send + Sync {
    fn read(&self, &str) -> Vec<Metric>;
}

pub struct StandardLogLineReader;

lazy_static! {
    static ref LOG_MEASURE_REGEX: Regex =
        Regex::new(r"measure#([a-zA-Z0-9._]+)=(\d+(?:\.\d+))").unwrap();

    static ref LOG_COUNT_REGEX: Regex =
        Regex::new(r"count#([a-zA-Z0-9._]+)=(\d+)").unwrap();
}

impl LogLineReader for StandardLogLineReader {
    fn read(&self, line: &str) -> Vec<Metric> {
        let mut metrics = vec![];

        // Look for measures
        for cap in LOG_MEASURE_REGEX.captures_iter(line) {
            let name = cap.at(1).unwrap();

            if let Ok(value) = f64::from_str(cap.at(2).unwrap()) {
                metrics.push(Measure(name.to_owned(), value))
            }
        }

        // Look for counts
        for cap in LOG_COUNT_REGEX.captures_iter(line) {
            let name = cap.at(1).unwrap();

            if let Ok(value) = u64::from_str(cap.at(2).unwrap()) {
                metrics.push(Count(name.to_owned(), value))
            }
        }

        metrics
    } // fn read
}
