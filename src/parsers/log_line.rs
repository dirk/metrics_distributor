use regex::{Regex};
use std::str::{FromStr};

use super::super::metrics::*;

/// Reader that takes a log line string and returns any metrics found in it.
pub trait LogLineReader: Send + Sync {
    fn read(&self, line: &str) -> Vec<Metric>;
}

/// Reads metrics from log lines in the standard formats:
///
/// - Measures: `measure#metric=1.2`
/// - Counts: `count#metric=3`
pub struct StandardLogLineReader;

lazy_static! {
    static ref LOG_MEASURE_REGEX: Regex =
        Regex::new(r"measure#([[:alnum:]._]+)=(\d+(?:\.\d+)?)").unwrap();

    static ref LOG_SAMPLE_REGEX: Regex =
        Regex::new(r"sample#([[:alnum:]._]+)=(\d+(?:\.\d+)?)").unwrap();

    static ref LOG_COUNT_REGEX: Regex =
        Regex::new(r"count#([[:alnum:]._]+)=(\d+)").unwrap();

    static ref SOURCE_REGEX: Regex =
        Regex::new(r"source=([[:alnum:]._]+)").unwrap();
}

impl StandardLogLineReader {
    fn parse_source(line: &str) -> Option<&str> {
        SOURCE_REGEX.captures(line)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str())
    }
}

impl LogLineReader for StandardLogLineReader {
    fn read(&self, line: &str) -> Vec<Metric> {
        let source = StandardLogLineReader::parse_source(line).map(|s| s.to_owned());
        let dimension = |name: &str| {
            Dimension { name: name.to_owned(), source: source.clone() }
        };

        let mut metrics = vec![];

        // Look for counts
        for cap in LOG_COUNT_REGEX.captures_iter(line) {
            let name = cap.get(1).unwrap().as_str();

            if let Ok(value) = u64::from_str(cap.get(2).unwrap().as_str()) {
                metrics.push(Count(dimension(name), value))
            }
        }

        // Look for measures
        for cap in LOG_MEASURE_REGEX.captures_iter(line) {
            let name = cap.get(1).unwrap().as_str();

            if let Ok(value) = f64::from_str(cap.get(2).unwrap().as_str()) {
                metrics.push(Measure(dimension(name), value))
            }
        }

        // Look for samples
        for cap in LOG_SAMPLE_REGEX.captures_iter(line) {
            let name = cap.get(1).unwrap().as_str();

            if let Ok(value) = f64::from_str(cap.get(2).unwrap().as_str()) {
                metrics.push(Sample(dimension(name), value))
            }
        }

        metrics
    } // fn read
}

/// Reads Heroku's logging metrics.
pub struct HerokuLogLineReader;

lazy_static! {
    static ref DYNO_TYPE_REGEX: Regex =
        Regex::new(r"dyno=([[:alpha:]]+)").unwrap();

    static ref CONNECT_REGEX: Regex =
        Regex::new(r"connect=(\d+)ms").unwrap();

    static ref SERVICE_REGEX: Regex =
        Regex::new(r"service=(\d+)ms").unwrap();

    static ref STATUS_REGEX: Regex =
        Regex::new(r"status=(\d+)").unwrap();

    static ref HEROKU_HTTP_ERROR_CODE_REGEX: Regex =
        Regex::new(r"code=(H\d+)").unwrap();

    static ref HEROKU_RUNTIME_ERROR_CODE_REGEX: Regex =
        Regex::new(r"Error (R\d+)").unwrap();

    static ref LOAD_AVG_1M_REGEX: Regex =
        Regex::new(r"sample#load_avg_1m=([0-9.]+)").unwrap();
}

impl HerokuLogLineReader {
    /// Parses Heroku router status lines for the response service time (how
    /// long it took) and the HTTP response status.
    pub fn parse_status(line: &str) -> Option<Vec<Metric>> {
        let mut metrics: Vec<Metric> = vec![];

        let connect = match CONNECT_REGEX.captures(line)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
            .and_then(|c| u16::from_str(c).ok()) {
                Some(c) => c,
                None => return None,
            };

        let status = match STATUS_REGEX.captures(line)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
            .and_then(|s| u16::from_str(s).ok()) {
                Some(s) => s,
                None => return None,
            };

        let service = match SERVICE_REGEX.captures(line)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
            .and_then(|s| u32::from_str(s).ok()) {
                Some(s) => s,
                None => return None,
            };

        let dyno_type = match DYNO_TYPE_REGEX.captures(line)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str()) {
                Some(d) => d,
                None => return None,
            };

        let base = format!("dyno.{}", dyno_type);

        // Counting a 499 as a 500
        let is_500 = status >= 499 && status < 600;

        // Don't record timing for 499 and 5xx errors
        if !is_500 {
            let service_time_name = format!("{}.service_time", base);
            metrics.push(Measure(Dimension::with_name_and_source(service_time_name, dyno_type), service as f64));
        }

        // Track the connect time (how long it took to pick up the request)
        let connect_time_name = format!("{}.connect_time", base);
        metrics.push(Measure(Dimension::with_name_and_source(connect_time_name, dyno_type), connect as f64));

        // Count the status
        let status_name = format!("{}.status.{}", base, status);
        metrics.push(Count(Dimension::with_name_and_source(status_name, dyno_type), 1));

        Some(metrics)
    }

    /// Parses Heroku warning and error codes like "Hxx" and "Rxx" where "xx" is a pair
    /// of numbers. See the [Heroku][] site for more details.
    ///
    /// [Heroku]: https://devcenter.heroku.com/articles/error-codes
    pub fn parse_heroku_code(line: &str) -> Option<Metric> {
        let code: &str;

        if let Some(http_code) = HEROKU_HTTP_ERROR_CODE_REGEX.captures(line).and_then(|c| c.get(1)).map(|m| m.as_str()) {
            code = http_code;

        } else if let Some(runtime_code) = HEROKU_RUNTIME_ERROR_CODE_REGEX.captures(line).and_then(|c| c.get(1)).map(|m| m.as_str()) {
            code = runtime_code;

        } else {
            return None
        }

        let name = format!("heroku.error.{}", code);
        Some(Count(Dimension::with_name(name), 1))
    }

    /// Parses the `sample#load_avg_1m=` metrics from Heroku logs.
    pub fn parse_load(line: &str) -> Option<Metric> {
        let load_avg_1m = match LOAD_AVG_1M_REGEX.captures(line)
                                                 .and_then(|c| c.get(1))
                                                 .map(|m| m.as_str())
                                                 .and_then(|c| f64::from_str(c).ok()) {
            Some(l) => l,
            None => return None,
        };

        let source = match StandardLogLineReader::parse_source(line) {
            Some(s) => s,
            None => return None,
        };

        let dyno_type = match source.split('.').nth(0) {
            Some(s) => s,
            None => return None,
        };

        let dim = Dimension::with_name_and_source(format!("dyno.{}.load_avg_1m", dyno_type), source);
        Some(Measure(dim, load_avg_1m))
    }
}

impl LogLineReader for HerokuLogLineReader {
    fn read(&self, line: &str) -> Vec<Metric> {
        if !line.contains("heroku") { return vec![] }

        let mut metrics: Vec<Metric> = vec![];

        if let Some(statuses) = HerokuLogLineReader::parse_status(line)      { metrics.extend(statuses) }
        if let Some(code)     = HerokuLogLineReader::parse_heroku_code(line) { metrics.push(code) }
        if let Some(load)     = HerokuLogLineReader::parse_load(line)        { metrics.push(load) }

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
            vec![ Measure(Dimension::with_name("foo"), 1.2) ]
        )
    }

    #[test]
    fn standard_reader_reads_measure_with_source() {
        let reader = StandardLogLineReader;
        let line = "source=web measure#foo=1.2\n";

        assert_eq!(
            reader.read(line),
            vec![ Measure(Dimension::with_name_and_source("foo", "web"), 1.2) ]
        )
    }

    #[test]
    fn standard_reader_reads_count() {
        let reader = StandardLogLineReader;
        let line = "count#foo=1\n";

        assert_eq!(
            reader.read(line),
            vec![ Count(Dimension::with_name("foo"), 1) ]
        )
    }

    #[test]
    fn standard_reader_reads_sample() {
        let reader = StandardLogLineReader;
        let line = "sample#bar=3.4\n";

        assert_eq!(
            reader.read(line),
            vec![ Sample(Dimension::with_name("bar"), 3.4) ]
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
    fn standard_reader_parses_source() {
        let line = "source=something metric#other=5.6\n";

        assert_eq!(
            StandardLogLineReader::parse_source(line),
            Some("something")
        )
    }

    #[test]
    fn heroku_reader_reads_loads() {
        let reader = HerokuLogLineReader;
        let line = "2016-02-26 21:34:59.429615+00:00 heroku web.2 - - source=web.2 dyno=heroku.123.XYZ sample#load_avg_1m=0.56 sample#load_avg_5m=0.26 sample#load_avg_15m=0.17\n";

        assert_eq!(
            reader.read(line),
            vec![ Measure(Dimension::with_name_and_source("dyno.web.load_avg_1m", "web.2"), 0.56) ]
        )
    }

    #[test]
    fn heroku_reader_reads_http_errors() {
        let reader = HerokuLogLineReader;
        let line = "2016-02-26 21:50:36.352129+00:00 heroku router - - sock=backend at=error code=H18 desc=\"Server Request Interrupted\" method=GET path=\"/\" host=www.example.com request_id=XYZ fwd=\"1.2.3.4\" dyno=web.5 connect=0ms service=495ms status=503 bytes=1648\n";

        assert_eq!(
            reader.read(line),
            vec![
                Measure(Dimension::with_name_and_source("dyno.web.connect_time", "web"), 0.0),
                Count(Dimension::with_name_and_source("dyno.web.status.503", "web"), 1),
                Count(Dimension::with_name("heroku.error.H18"), 1),
            ]
        )
    }

    #[test]
    fn heroku_reader_reads_runtime_errors() {
        let reader = HerokuLogLineReader;
        let line = "2016-02-25 21:35:34.990292+00:00 heroku scheduler.5451 - - Error R14 (Memory quota exceeded)\n";

        assert_eq!(
            reader.read(line),
            vec![
                Count(Dimension::with_name("heroku.error.R14"), 1),
            ]
        )
    }

    #[test]
    fn heroku_reader_reads_service_times() {
        let reader = HerokuLogLineReader;
        let line = "2016-02-26 21:34:59.370813+00:00 heroku router - - at=info method=PUT path=\"/\" host=www.example.com request_id=XYZ fwd=\"1.2.3.4\" dyno=web.1 connect=1ms service=39ms status=200 bytes=1627\n";

        assert_eq!(
            reader.read(line),
            vec![
                Measure(Dimension::with_name_and_source("dyno.web.service_time", "web"), 39.0),
                Measure(Dimension::with_name_and_source("dyno.web.connect_time", "web"), 1.0),
                Count(Dimension::with_name_and_source("dyno.web.status.200", "web"), 1),
            ]
        )
    }
}
