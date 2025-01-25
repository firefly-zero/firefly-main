#![no_std]
#![allow(clippy::new_without_default)]
extern crate alloc;
mod display;
mod error;
pub use display::{Display3, Writer};
pub use error::*;
