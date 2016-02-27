extern crate metrics_distributor;

use metrics_distributor::SharedStore;
use metrics_distributor::collectors::statsd::StatsdUdpListener;

fn main() {
    let store = SharedStore::new();

    let listener = StatsdUdpListener::new(store);
    listener.listen("localhost:9876");
}
