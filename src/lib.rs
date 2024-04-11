#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

pub mod contract;
pub mod errors;
pub mod price_data;
pub mod storage;
pub mod types;

#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

#[cfg(test)]
mod tests;
