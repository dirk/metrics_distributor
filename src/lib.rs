#[macro_use]
extern crate lazy_static;

extern crate hyper;
extern crate iron;
extern crate regex;

pub mod distributor;
pub mod metrics;
pub mod reader;
pub mod server;

pub use distributor::*;

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
