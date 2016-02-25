extern crate metrics_distributor;

use metrics_distributor::SharedStore;
use metrics_distributor::collectors::statsd::StatsdTcpListener;

fn main() {
    let listener = StatsdTcpListener::new();
    listener.listen("localhost:9876");
}
