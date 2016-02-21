#[macro_use]
extern crate lazy_static;

extern crate hyper;
extern crate iron;
extern crate regex;

pub mod store;
pub mod metrics;
pub mod reader;

/// Tools for building collectors to be exposed through the Iron HTTP library.
pub mod http;

pub use store::*;

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
