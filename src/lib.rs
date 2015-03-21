// Rust language wrapper for Termbox.
// Termbox is available under the MIT license.
// These bindings are public domain.

#![crate_name="termbox"]
#![crate_type="lib"]

#![feature(core)]

#[macro_use]
extern crate bitflags;
extern crate libc;

pub use self::public::*;

pub mod public;

#[allow(dead_code)]
mod ffi;
#[cfg(test)]
mod test;
