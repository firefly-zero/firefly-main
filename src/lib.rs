#![no_std]
#![allow(clippy::new_without_default)]
extern crate alloc;
mod display;
mod error;
pub use display::{Display, Writer};
pub use error::*;
