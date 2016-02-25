extern crate metrics_distributor;

use metrics_distributor::SharedStore;
use metrics_distributor::collectors::statsd::StatsdTcpListener;

fn main() {
    let store = SharedStore::new();

    let listener = StatsdTcpListener::new(store);
    listener.listen("localhost:9876");
}
