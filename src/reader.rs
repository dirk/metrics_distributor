use regex::{Regex};
use std::str::{FromStr};

use super::Distributor;

trait LogLineReader {
    fn read(&self, &mut Distributor, &str);
}

struct StandardLogLineReader;

lazy_static! {
    static ref LOG_MEASURE_REGEX: Regex =
        Regex::new(r"measure#([a-zA-Z0-9._]+)=(\d+)").unwrap();

    static ref LOG_COUNT_REGEX: Regex =
        Regex::new(r"count#([a-zA-Z0-9._]+)=(\d+)").unwrap();
}

impl LogLineReader for StandardLogLineReader {
    fn read(&self, dist: &mut Distributor, line: &str) {

        // Look for measures
        for cap in LOG_MEASURE_REGEX.captures_iter(line) {
            let name = cap.at(1).unwrap();

            if let Ok(value) = f64::from_str(cap.at(2).unwrap()) {
                dist.record_measure(name.to_owned(), value)
            }
        }

        // Look for counts
        for cap in LOG_COUNT_REGEX.captures_iter(line) {
            let name = cap.at(1).unwrap();

            if let Ok(value) = u64::from_str(cap.at(2).unwrap()) {
                dist.record_count(name.to_owned(), value)
            }
        }

    } // fn read
}
