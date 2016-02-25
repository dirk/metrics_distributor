use std::char;
use std::error;
use std::fmt;
use std::num::ParseIntError;
use std::str::{self, FromStr};

use nom::{
    digit,
    is_alphanumeric,
    IResult
};

use super::super::metrics::Metric;

/// Parsed StatsD metric.
///
/// See here for more details: https://github.com/b/statsd_spec#metric-types--formats
#[derive(Debug, PartialEq)]
pub enum ParsedMetric {
    Counter(String, u64),
    Gauge(String, u64),
    Timer(String, u64),
}

impl ParsedMetric {
    pub fn to_standard_metric(&self) -> Metric {
        use self::ParsedMetric::*;

        match self {
            &Counter(ref name, value) => Metric::Count(name.clone(), value),
            &Gauge(ref name, value)   => Metric::Sample(name.clone(), value as f64),
            &Timer(ref name, value)   => Metric::Measure(name.clone(), value as f64),
        }
    }
}

pub type ParseResult<'a> = IResult<&'a [u8], ParsedMetric>;

// Convert a byte array to a `u64` result.
fn bytes_to_u64(i: &[u8]) -> Result<u64, ParseIntError> {
    let s = str::from_utf8(i).unwrap();

    u64::from_str(s)
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    description: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        &self.description
    }
}

pub fn parse_metrics(i: &[u8]) -> Result<Vec<ParsedMetric>, ParseError> {
    let result = complete!(i,
        separated_nonempty_list!(
            tag!("\n"),
            alt_complete!(
                parse_counter |
                parse_gauge |
                parse_timer
            )
        )
    );

    match result {
        IResult::Done(_, metrics) => Ok(metrics),
        IResult::Error(err) => Err(ParseError { description: format!("{:?}", err) }),
        IResult::Incomplete(_) => unreachable!("Cannot be Incomplete"),
    }
}

pub fn parse_counter(i: &[u8]) -> ParseResult {
    chain!(i,
        name: parse_metric_name ~ tag!(":")  ~
        value: parse_value      ~ tag!("|c") ~
        _sample_rate: opt!(complete!(parse_sample_rate)) ,

        ||{ ParsedMetric::Counter(name, value) }
    )
}

pub fn parse_gauge(i: &[u8]) -> ParseResult {
    chain!(i,
        name: parse_metric_name ~ tag!(":")  ~
        value: parse_value      ~ tag!("|g") ,

        ||{ ParsedMetric::Gauge(name, value) }
    )
}

pub fn parse_timer(i: &[u8]) -> ParseResult {
    chain!(i,
        name: parse_metric_name ~ tag!(":")  ~
        value: parse_value      ~ tag!("|ms") ,

        ||{ ParsedMetric::Timer(name, value) }
    )
}

pub fn parse_value(i: &[u8]) -> IResult<&[u8], u64> {
    map_res!(i,
        digit,
        |value| { bytes_to_u64(value) }
    )
}

pub fn parse_sample_rate(i: &[u8]) -> IResult<&[u8], u64> {
    preceded!(i,
        tag!("|@"),
        map_res!(digit, |rate| { bytes_to_u64(rate) })
    )
}

fn parse_metric_name(i: &[u8]) -> IResult<&[u8], String> {
    #[inline]
    fn is_metric_name_char(i: u8) -> bool {
        let c = char::from_u32(i as u32).unwrap();

        is_alphanumeric(i) || c == '.' || c == '_'
    }

    map!(i,
        take_while!(is_metric_name_char),
        |name| { String::from_utf8_lossy(name).into_owned() }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::any::Any;
    use nom::IResult;

    fn complete<'a, T>(value: T) -> IResult<&'a [u8], T>
        where T: Any {
        IResult::Done(&b""[..], value)
    }

    #[test]
    fn it_parses_gauge() {
        assert_eq!(
            parse_gauge(&b"foo.bar_baz:12|g"[..]),
            complete(ParsedMetric::Gauge("foo.bar_baz".to_owned(), 12))
        )
    }

    #[test]
    fn it_parses_counter() {
        assert_eq!(
            parse_counter(&b"foo.bar_baz:23|c"[..]),
            complete(ParsedMetric::Counter("foo.bar_baz".to_owned(), 23))
        )
    }

    #[test]
    fn it_parses_counter_with_sample_rate() {
        assert_eq!(
            parse_counter(&b"foo.bar_baz:34|c|@5"[..]),
            complete(ParsedMetric::Counter("foo.bar_baz".to_owned(), 34))
        )
    }

    #[test]
    fn it_parses_timer() {
        assert_eq!(
            parse_timer(&b"foo.bar_baz:12|ms"[..]),
            complete(ParsedMetric::Timer("foo.bar_baz".to_owned(), 12))
        )
    }

    #[test]
    fn it_parse_single_metric() {
        assert_eq!(
            parse_metrics(&b"foo:1|g"[..]),
            Ok(vec![
                ParsedMetric::Gauge("foo".to_owned(), 1),
            ])
        )
    }

    #[test]
    fn it_parse_many_metrics() {
        assert_eq!(
            parse_metrics(&b"foo:1|g\nbar:2|c|@3\nbaz:4|ms"[..]),
            Ok(vec![
                ParsedMetric::Gauge("foo".to_owned(), 1),
                ParsedMetric::Counter("bar".to_owned(), 2),
                ParsedMetric::Timer("baz".to_owned(), 4),
            ])
        )
    }
}
