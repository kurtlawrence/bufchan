#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod chan;
#[cfg(test)]
mod test;

pub use chan::{unbounded, unbounded_with_buffer, Receiver, Sender};
