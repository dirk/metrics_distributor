#[macro_use]
extern crate lazy_static;

extern crate iron;
extern crate regex;

mod distributor;
mod metrics;
mod reader;
mod server;

pub use distributor::Distributor;

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
