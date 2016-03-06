#![feature(test)]

extern crate metrics_distributor;
extern crate test;

use metrics_distributor::parsers::log_line::*;
use metrics_distributor::store::BaseStore;
use test::Bencher;

#[bench]
fn bench_heroku_log_line_reader(b: &mut Bencher) {
    let lines = vec![
        "2016-02-26 21:34:59.429615+00:00 heroku web.2 - - source=web.2 dyno=heroku.123.XYZ sample#load_avg_1m=0.56 sample#load_avg_5m=0.26 sample#load_avg_15m=0.17\n",
        "2016-02-26 21:50:36.352129+00:00 heroku router - - sock=backend at=error code=H18 desc=\"Server Request Interrupted\" method=GET path=\"/\" host=www.example.com request_id=XYZ fwd=\"1.2.3.4\" dyno=web.5 connect=0ms service=495ms status=503 bytes=1648\n",
        "2016-02-26 21:34:59.370813+00:00 heroku router - - at=info method=PUT path=\"/\" host=www.example.com request_id=XYZ fwd=\"1.2.3.4\" dyno=web.1 connect=0ms service=39ms status=200 bytes=1627\n",
    ];

    b.iter(|| {
        lines.iter().map(|line| {
            HerokuLogLineReader.read(line)
        }).collect::<Vec<_>>()
    })
}

#[bench]
fn bench_insert_and_aggregate(b: &mut Bencher) {
    use metrics_distributor::metrics::Metric::*;

    let metrics = vec![
        Count("a".to_owned(), 1),
        Count("a".to_owned(), 2),
        Count("a".to_owned(), 3),
        Measure("b".to_owned(), 0.1),
        Measure("b".to_owned(), 0.2),
        Measure("b".to_owned(), 0.3),
        Sample("c".to_owned(), 0.4),
        Sample("c".to_owned(), 0.5),
        Sample("c".to_owned(), 0.6),
    ];

    b.iter(|| {
        let mut store = BaseStore::new();

        store.record(metrics.clone());

        store.flush()
    })
}
