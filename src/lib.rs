#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

pub mod contract;
mod errors;
mod price_data;
mod storage;
mod types;

pub use contract::*;
#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

#[cfg(test)]
mod tests;
