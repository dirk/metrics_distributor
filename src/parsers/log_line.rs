use regex::{Regex};
use std::str::{FromStr};

use super::super::metrics::*;

/// Reader that takes a log line string and returns any metrics found in it.
pub trait LogLineReader: Send + Sync {
    fn read(&self, &str) -> Vec<Metric>;
}

/// Reads metrics from log lines in the standard formats:
///
/// - Measures: `measure#metric=1.2`
/// - Counts: `count#metric=3`
pub struct StandardLogLineReader;

lazy_static! {
    static ref LOG_MEASURE_REGEX: Regex =
        Regex::new(r"measure#([a-zA-Z0-9._]+)=(\d+(?:\.\d+)?)").unwrap();

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

#[cfg(test)]
mod tests {
    use super::{LogLineReader, StandardLogLineReader};
    use super::super::metrics::*;

    #[test]
    fn it_reads_measure() {
        let reader = StandardLogLineReader;
        let line = "measure#foo=1.2\n";

        assert_eq!(
            reader.read(line),
            vec![Measure("foo".to_owned(), 1.2)]
        )
    }

    #[test]
    fn it_reads_count() {
        let reader = StandardLogLineReader;
        let line = "count#foo=1\n";

        assert_eq!(
            reader.read(line),
            vec![Count("foo".to_owned(), 1)]
        )
    }

    #[test]
    fn it_returns_nothing_on_failed_read() {
        let reader = StandardLogLineReader;
        let line = "metric#bar=3.4\n";

        assert_eq!(
            reader.read(line),
            vec![]
        )
    }
}
