use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

use super::super::SharedStore;
use super::super::parsers::statsd::parse_metrics;

/// Listens on a TCP socket for StatsD messages.
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
            StatsdTcpListener::accept_on_listener(listener, send)
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
            Err(err) => {
                println!("{:?}", err)
            },
        }
    }

    fn accept_on_listener(listener: TcpListener, send: Sender<String>) {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    // Clients have 30 seconds to send us data before we'll drop.
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));

                    let send = send.clone();

                    thread::spawn(move || {
                        StatsdTcpListener::handle_client(stream, send)
                    });
                },
                Err(e) => panic!("Failed to listen on TCP socket: {}", e),
            }
        }
    }

    fn handle_client(stream: TcpStream, send: Sender<String>) {
        let mut reader = BufReader::new(stream);

        loop {
            let mut line = String::new();

            match reader.read_line(&mut line) {
                Err(err) => {
                    println!("Error reading StatsD line: {:?}", err);
                    break
                },
                Ok(0) => {
                    // Close if there are no more bytes.
                    break
                },
                Ok(_) => {
                    send.send(line).unwrap()
                },
            }
        }
    }

}
