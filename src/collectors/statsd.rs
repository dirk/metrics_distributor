use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::u64;

pub struct StatsdListener {
    listener: TcpListener,
}

impl StatsdListener {
    pub fn listen(&self) {
        let (send, recv) = channel();

        let listener = self.listener.try_clone().unwrap();
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
