use std::io::{BufRead, BufReader};
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::u64;

use super::super::parsers::statsd::parse_metrics;

pub struct StatsdTcpListener;

impl StatsdTcpListener {
    pub fn new() -> StatsdTcpListener {
        StatsdTcpListener
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

    }
}

fn accept_on_listener(listener: TcpListener, send: Sender<String>) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();

                reader.read_line(&mut line);
                send.send(line).unwrap();
            },
            Err(e) => panic!("Failed to listen on TCP socket: {}", e),
        }
    }
}
