#![cfg_attr(not(test), no_std)]

mod memvar_macro;

pub mod time;
pub mod sync;
pub mod var;
pub mod async_util;

#[macro_use]
pub mod print;

// re-export futures
pub use futures;
