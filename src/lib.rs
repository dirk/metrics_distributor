#[macro_use]
extern crate lazy_static;

extern crate regex;

mod distributor;
mod reader;
mod metrics;

pub use distributor::Distributor;

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
