//! A _high throughput_ and _simple_ buffered MPSC channel.
//!
//! Take at look at the [README](https://github.com/kurtlawrence/bufchan/blob/main/README.md) for benchmarks and caveats.
//!
//! # Example
//! ```rust
//! let (mut tx, rx) = bufchan::unbounded();
//!
//! std::thread::spawn(move || {
//!     (1..=10).for_each(|x| tx.send(x));
//! });
//!
//! assert_eq!(
//!     rx.into_iter().collect::<Vec<_>>(),
//!     vec![1,2,3,4,5,6,7,8,9,10]
//! );
//! ```
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod chan;
#[cfg(test)]
mod test;

pub use chan::{unbounded, unbounded_with_buffer, Receiver, Sender};
