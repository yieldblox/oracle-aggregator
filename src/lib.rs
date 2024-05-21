#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

pub mod contract;
mod errors;
mod storage;

#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

#[cfg(test)]
mod tests;
