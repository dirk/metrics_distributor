extern crate metrics_distributor;
extern crate iron;
extern crate router;

use metrics_distributor::SharedStore;
use metrics_distributor::collectors::http::LogDrainHandler;
use metrics_distributor::parsers::log_line::StandardLogLineReader;
use iron::prelude::*;
use router::Router;

fn main() {
    let store = SharedStore::new();

    let log_drain = LogDrainHandler::new(store, vec![
        Box::new(StandardLogLineReader)
    ]);

    let mut router = Router::new();
    router.post("/logs/drain", log_drain);

    Iron::new(router).http("localhost:3000").unwrap();
}
