use std::char;
use std::str::{self, FromStr};

use nom::{
    digit,
    is_alphanumeric,
    IResult
};

/// Parsed StatsD metric.
///
/// See here for more details: https://github.com/b/statsd_spec#metric-types--formats
#[derive(Debug, PartialEq)]
pub enum ParsedMetric {
    Gauge(String, u64),
}

pub type ParseResult<'a> = IResult<&'a [u8], ParsedMetric>;

fn bytes_to_u64(i: &[u8]) -> u64 {
    let s = str::from_utf8(i).unwrap();

    u64::from_str(s).unwrap()
}

pub fn parse_gauge(i: &[u8]) -> ParseResult {
    chain!(i,
        name: parse_metric_name ~ tag!(":")  ~
        value: digit            ~ tag!("|g") ,

        ||{ ParsedMetric::Gauge(name, bytes_to_u64(value)) }
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
            parse_gauge(&b"foo.bar_baz:1|g"[..]),
            complete(ParsedMetric::Gauge("foo.bar_baz".to_owned(), 1))
        )
    }
}
