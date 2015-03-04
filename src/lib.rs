// Rust language wrapper for Termbox.
// Termbox was written in C by nsf <no.smile.face@gmail.com>.
// Project page: https://code.google.com/p/termbox/
// This binding is subject to the terms of the original library.

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
