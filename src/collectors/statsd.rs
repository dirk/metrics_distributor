use std::io::{BufRead, BufReader};
use std::str;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

use std::net::{
    TcpListener,
    TcpStream,
    ToSocketAddrs,
    UdpSocket
};

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
            handle_line(&self.store, line)
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
    } // fn handle_client
} // struct StatsdTcpListener

/// Listens for StatsD UDP datagrams.
pub struct StatsdUdpListener {
    store: SharedStore,
}

impl StatsdUdpListener {
    pub fn new(store: SharedStore) -> StatsdUdpListener {
        StatsdUdpListener {
            store: store,
        }
    }

    pub fn listen<A>(&self, addr: A)
        where A: ToSocketAddrs {
        let (send, recv) = channel();

        let socket = UdpSocket::bind(addr).unwrap();

        thread::spawn(move || {
            let mut buf = [0; 1024];
            loop {
                let (bytes_read, _) = match socket.recv_from(&mut buf) {
                    Ok(pair) => pair,
                    Err(_) => return,
                };

                // Get a string from just the amount of bytes read.
                let message: &str = match str::from_utf8(&buf[..bytes_read]) {
                    Ok(s) => s,
                    Err(_) => return,
                };

                send.send(message.to_owned()).unwrap();
            }
        });

        for line in recv {
            handle_line(&self.store, line)
        }
    } // fn listen
} // impl StatsdUdpListener

fn handle_line(store: &SharedStore, line: String) {
    let line_trimmed = line.trim_right();
    let result = parse_metrics(line_trimmed.as_bytes());

    match result {
        Ok(metrics) => {
            let metrics = metrics.iter().map(|m| m.to_standard_metric()).collect();
            store.record(metrics)
        },
        Err(err) => {
            println!("{:?}", err)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::handle_line;
    use super::super::super::SharedStore;
    use super::super::super::metrics::{AggregatedMetrics, AggregatedMetricType};

    #[test]
    fn handle_line_parses_metrics() {
        let store = SharedStore::new();
        handle_line(&store, "foo:1|g".to_owned());

        assert_eq!(store.flush(), AggregatedMetrics::with_metrics(vec![
            (AggregatedMetricType::Sample, "foo".to_owned(), 1.0),
        ]));
    }
}
