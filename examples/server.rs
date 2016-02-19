extern crate metrics_distributor;
extern crate iron;
extern crate router;

use metrics_distributor::SharedDistributor;
use metrics_distributor::server::LogDrainHandler;
use metrics_distributor::reader::StandardLogLineReader;
use iron::prelude::*;
use router::Router;

fn main() {
    let distributor = SharedDistributor::new();

    let log_drain = LogDrainHandler::new(distributor, vec![
        Box::new(StandardLogLineReader)
    ]);

    let mut router = Router::new();
    router.post("/logs/drain", log_drain);

    Iron::new(router).http("localhost:3000").unwrap();
}
