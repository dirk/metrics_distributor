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

/// Reads Heroku's logging metrics.
pub struct HerokuLogLineReader;

lazy_static! {
    static ref DYNO_TYPE_REGEX: Regex =
        Regex::new(r"dyno=(\w+)").unwrap();

    static ref SERVICE_REGEX: Regex =
        Regex::new(r"service=(\d+)ms").unwrap();

    static ref STATUS_REGEX: Regex =
        Regex::new(r"status=(\d+)").unwrap();

    static ref HEROKU_ERROR_CODE_REGEX: Regex =
        Regex::new(r"code=(H\d+)").unwrap();

    static ref LOAD_AVG_1M_REGEX: Regex =
        Regex::new(r"sample#load_avg_1m=([0-9.]+)").unwrap();

    static ref SOURCE_REGEX: Regex =
        Regex::new(r"source=(\w+).(\d+)").unwrap();
}

impl HerokuLogLineReader {
    /// Parses Heroku router status lines for the response service time (how
    /// long it took) and the HTTP response status.
    pub fn parse_status(line: &str) -> Vec<Metric> {
        let mut metrics: Vec<Metric> = vec![];

        let status = match STATUS_REGEX.captures(line) {
            Some(captures) => captures.at(1).unwrap(),
            None => return vec![],
        };
        let status = u16::from_str(status).unwrap();

        let service = SERVICE_REGEX.captures(line)
            .and_then(|c| c.at(1))
            .and_then(|s| u32::from_str(s).ok())
            .unwrap();

        let dyno_type = DYNO_TYPE_REGEX.captures(line).and_then(|c| c.at(1)).unwrap();

        let base = format!("dyno.{}", dyno_type);

        // Don't record timing for 499 and 5xx errors
        if status < 499 || status > 599 {
            let service_time = format!("{}.service_time", base);
            metrics.push(Measure(service_time.to_owned(), service as f64));
        }

        // Count the status
        metrics.push(Count(format!("{}.status.{}", base, status).to_owned(), 1));

        metrics
    }

    /// Parses Heroku warning and error codes like "Hxx" where "xx" is a pair
    /// of numbers. See the [Heroku][] site for more details.
    ///
    /// [Heroku]: https://devcenter.heroku.com/articles/error-codes
    pub fn parse_heroku_codes(line: &str) -> Vec<Metric> {
        let is_warning = line.contains("at=warning");
        let is_error   = line.contains("at=error");

        if !is_warning && !is_error { return vec![] }

        let code = HEROKU_ERROR_CODE_REGEX.captures(line).and_then(|c| c.at(1)).unwrap();

        vec![
            Count(format!("heroku.error.{}", code), 1)
        ]
    }

    /// Parses the `sample#load_avg_1m=` metrics from Heroku logs.
    pub fn parse_loads(line: &str) -> Vec<Metric> {
        let load_avg_1m = match LOAD_AVG_1M_REGEX.captures(line) {
            Some(captures) => captures.at(1).and_then(|c| f64::from_str(c).ok()).unwrap(),
            None => return vec![],
        };

        let dyno_type = SOURCE_REGEX.captures(line).and_then(|c| c.at(1)).unwrap();

        vec![
            Measure(format!("dyno.{}.load_avg_1m", dyno_type), load_avg_1m)
        ]
    }
}

impl LogLineReader for HerokuLogLineReader {
    fn read(&self, line: &str) -> Vec<Metric> {
        let mut metrics: Vec<Metric> = vec![];

        metrics.extend(HerokuLogLineReader::parse_status(line));
        metrics.extend(HerokuLogLineReader::parse_heroku_codes(line));
        metrics.extend(HerokuLogLineReader::parse_loads(line));

        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LogLineReader,
        StandardLogLineReader,
        HerokuLogLineReader
    };
    use super::super::super::metrics::*;

    #[test]
    fn standard_reader_reads_measure() {
        let reader = StandardLogLineReader;
        let line = "measure#foo=1.2\n";

        assert_eq!(
            reader.read(line),
            vec![ Measure("foo".to_owned(), 1.2) ]
        )
    }

    #[test]
    fn standard_reader_reads_count() {
        let reader = StandardLogLineReader;
        let line = "count#foo=1\n";

        assert_eq!(
            reader.read(line),
            vec![ Count("foo".to_owned(), 1) ]
        )
    }

    #[test]
    fn standard_reader_returns_nothing_on_failed_read() {
        let reader = StandardLogLineReader;
        let line = "metric#bar=3.4\n";

        assert_eq!(
            reader.read(line),
            vec![]
        )
    }

    #[test]
    fn heroku_reader_reads_loads() {
        let reader = HerokuLogLineReader;
        let line = "2016-02-26 21:34:59.429615+00:00 heroku web.2 - - source=web.2 dyno=heroku.123.XYZ sample#load_avg_1m=0.56 sample#load_avg_5m=0.26 sample#load_avg_15m=0.17\n";

        assert_eq!(
            reader.read(line),
            vec![ Measure("dyno.web.load_avg_1m".to_owned(), 0.56) ]
        )
    }

    #[test]
    fn heroku_reader_reads_errors() {
        let reader = HerokuLogLineReader;
        let line = "2016-02-26 21:50:36.352129+00:00 heroku router - - sock=backend at=error code=H18 desc=\"Server Request Interrupted\" method=GET path=\"/\" host=www.example.com request_id=XYZ fwd=\"1.2.3.4\" dyno=web.5 connect=0ms service=495ms status=503 bytes=1648\n";

        assert_eq!(
            reader.read(line),
            vec![
                Count("dyno.web.status.503".to_owned(), 1),
                Count("heroku.error.H18".to_owned(), 1),
            ]
        )
    }

    #[test]
    fn heroku_reader_reads_service_times() {
        let reader = HerokuLogLineReader;
        let line = "2016-02-26 21:34:59.370813+00:00 heroku router - - at=info method=PUT path=\"/\" host=www.example.com request_id=XYZ fwd=\"1.2.3.4\" dyno=web.1 connect=0ms service=39ms status=200 bytes=1627\n";

        assert_eq!(
            reader.read(line),
            vec![
                Measure("dyno.web.service_time".to_owned(), 39.0),
                Count("dyno.web.status.200".to_owned(), 1),
            ]
        )
    }
}
