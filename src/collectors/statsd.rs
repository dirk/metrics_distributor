use std::io::{BufRead, BufReader};
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::mpsc::{channel, Sender};
use std::thread;

use super::super::SharedStore;
use super::super::parsers::statsd::parse_metrics;

pub struct StatsdTcpListener {
    store: SharedStore,
}

impl StatsdTcpListener {
    pub fn new(store: SharedStore) -> StatsdTcpListener {
        StatsdTcpListener {
            store: store,
        }
    }

    pub fn listen<A>(&self, addr: A)
        where A: ToSocketAddrs {
        let (send, recv) = channel();

        let listener = TcpListener::bind(addr).unwrap();
        thread::spawn(move || {
            accept_on_listener(listener, send)
        });

        for line in recv {
            self.handle_line(line)
        }
    }

    fn handle_line(&self, line: String) {
        let line_trimmed = line.trim_right();
        let result = parse_metrics(line_trimmed.as_bytes());

        match result {
            Ok(metrics) => {
                let metrics = metrics.iter().map(|m| m.to_standard_metric()).collect();
                self.store.record(metrics)
            },
            _ => (),
        }
    }
}

fn accept_on_listener(listener: TcpListener, send: Sender<String>) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();

                reader.read_line(&mut line).unwrap();
                send.send(line).unwrap();
            },
            Err(e) => panic!("Failed to listen on TCP socket: {}", e),
        }
    }
}
