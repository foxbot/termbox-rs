// Rust language wrapper for Termbox.
// Termbox was written in C by nsf <no.smile.face@gmail.com>.
// Project page: https://code.google.com/p/termbox/
// This binding is subject to the terms of the original library.

#![crate_name="termbox"]
#![crate_type="lib"]

#![feature(core)]
#![feature(hash)]

#[macro_use]
extern crate bitflags;
extern crate libc;

use std::cmp::min;
use std::fmt::{
  Display,
  Formatter,
};
use std::mem::zeroed;
use std::slice::{
  from_raw_parts,
  from_raw_parts_mut,
};
use std::sync::atomic::{
  AtomicBool,
  Ordering,
  ATOMIC_BOOL_INIT,
};
use libc::c_int;

#[allow(dead_code)]
mod ffi;

// global lock state
static mut _is_open: AtomicBool = ATOMIC_BOOL_INIT;

// public types
pub type Result<T> = std::result::Result<T, Error>;


//
// Cell
//


#[derive(Clone, Copy, Eq, Hash, PartialEq)]
#[repr(C)]
pub struct Cell {
  pub ch: char,
  pub fg: u16,
  pub bg: u16,
}


//
// Error
//


#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Error {
  FailedToOpenTty,
  PipeTrapError,
  PollFailed,
  TerminalLocked,
  Timeout,
  UnknownEvent,
  UnknownInitializationFailure,
  UnknownInputMode,
  UnknownOutputMode,
  UnsupportedTerminal,
}

impl Error {
  pub fn as_str (self) -> &'static str {
    match self {
      Error::FailedToOpenTty => "failed to open tty",
      Error::PipeTrapError => "pipe trap error",
      Error::PollFailed => "poll failed",
      Error::TerminalLocked => "terminal locked",
      Error::Timeout => "timeout",
      Error::UnknownEvent => "unknown event",
      Error::UnknownInitializationFailure => "unknown initialization failure",
      Error::UnknownInputMode => "unknown input mode",
      Error::UnknownOutputMode => "unknown output mode",
      Error::UnsupportedTerminal => "unsupported terminal",
    }
  }
}

impl Display for Error {
  fn fmt (&self, f: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
    self.as_str().fmt(f)
  }
}


//
// Event
//


#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Event {
  Key(KeySym),
  Resize(i32, i32),
}


//
// InputMode
//


#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum InputMode {
  Esc,
  Alt,
}

impl InputMode {
  fn from_c_int (mode: c_int) -> Option<InputMode> {
    match mode {
      ffi::TB_INPUT_ESC => Some(InputMode::Esc),
      ffi::TB_INPUT_ALT => Some(InputMode::Alt),
      _ => None
    }
  }

  fn to_c_int (self) -> c_int {
    match self {
      InputMode::Esc => ffi::TB_INPUT_ESC,
      InputMode::Alt => ffi::TB_INPUT_ALT,
    }
  }
}


//
// KeySum
//


#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct KeySym {
  pub mods: Mods,
  pub key: u16,
  pub ch: char,
}


//
// Mods
//


bitflags! {
  flags Mods: u8 {
    const MOD_ALT = 0x01,
  }
}


//
// OutputMode
//


#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum OutputMode {
  Normal,
  Color256,
  Color216,
  Grayscale,
}

impl OutputMode {
  fn from_c_int (mode: c_int) -> Option<OutputMode> {
    match mode {
      ffi::TB_OUTPUT_NORMAL => Some(OutputMode::Normal),
      ffi::TB_OUTPUT_256 => Some(OutputMode::Color256),
      ffi::TB_OUTPUT_216 => Some(OutputMode::Color216),
      ffi::TB_OUTPUT_GRAYSCALE => Some(OutputMode::Grayscale),
      _ => None
    }
  }

  fn to_c_int (self) -> c_int {
    match self {
      OutputMode::Normal => ffi::TB_OUTPUT_NORMAL,
      OutputMode::Color256 => ffi::TB_OUTPUT_256,
      OutputMode::Color216 => ffi::TB_OUTPUT_216,
      OutputMode::Grayscale => ffi::TB_OUTPUT_GRAYSCALE,
    }
  }
}


//
// Termbox
//


pub struct Termbox {
  #[allow(dead_code)]
  uninstantiable: (),
}

impl Termbox {
  pub fn blit (&mut self, x: i32, y: i32, w: i32, h: i32, cells: &[Cell]) {
    unsafe {
      if w > 0 && h > 0 {
        assert!(cells.len() >= (w * h) as usize);
      }

      // check dimensions
      let buffer_width = ffi::tb_width() as i32;
      let buffer_height = ffi::tb_height() as i32;
      if w < 1 || h < 1 || x + w <= 0 || y + h <= 0 || x >= buffer_width || y >= buffer_height {
        return;
      }

      // get valid bounds
      let min_x = if x < 0 {-x} else {0};
      let min_y = if y < 0 {-y} else {0};
      let max_x = min(x + w, buffer_width) - x;
      let max_y = min(y + h, buffer_height) - y;

      // blit
      let dst_ptr = ffi::tb_cell_buffer();
      let dst = from_raw_parts_mut(dst_ptr, (buffer_width * buffer_height) as usize);
      for cy in min_y..max_y {
        let mut src_index = (cy * w + min_x) as usize;
        let mut dst_index = ((y + cy) * buffer_width + x + min_x) as usize;
        for _ in min_x..max_x {
          dst[dst_index] = cells[src_index];
          src_index += 1;
          dst_index += 1;
        }
      }
    }
  }

  pub fn change_cell (&mut self, x: i32, y: i32, ch: char, fg: u16, bg: u16) {
    unsafe {
      ffi::tb_change_cell(x as c_int, y as c_int, ch as u32, fg, bg);
    }
  }

  pub fn clear (&mut self) {
    unsafe {
      ffi::tb_clear();
    }
  }

  pub fn get_cell_buffer (&self) -> &[Cell] {
    unsafe {
      let ptr = ffi::tb_cell_buffer() as *const Cell;
      let width = ffi::tb_width() as usize;
      let height = ffi::tb_height() as usize;
      return from_raw_parts(ptr, width * height);
    }
  }

  pub fn get_cell_buffer_mut (&self) -> &mut [Cell] {
    unsafe {
      let ptr = ffi::tb_cell_buffer();
      let width = ffi::tb_width() as usize;
      let height = ffi::tb_height() as usize;
      return from_raw_parts_mut(ptr, width * height);
    }
  }

  pub fn get_height (&self) -> i32 {
    unsafe {
      ffi::tb_height() as i32
    }
  }

  pub fn get_input_mode (&self) -> Result<InputMode> {
    unsafe {
      let mode = ffi::tb_select_input_mode(ffi::TB_INPUT_CURRENT);
      if let Some(mode) = InputMode::from_c_int(mode) {
        return Ok(mode);
      } else {
        return Err(Error::UnknownInputMode);
      }
    }
  }

  pub fn get_output_mode (&self) -> Result<OutputMode> {
    unsafe {
      let mode = ffi::tb_select_output_mode(ffi::TB_OUTPUT_CURRENT);
      if let Some(mode) = OutputMode::from_c_int(mode) {
        return Ok(mode);
      } else {
        return Err(Error::UnknownOutputMode);
      }
    }
  }

  pub fn get_width (&self) -> i32 {
    unsafe {
      ffi::tb_width() as i32
    }
  }

  pub fn hide_cursor (&mut self) {
    self.set_cursor(-1, -1);
  }

  pub fn new () -> Result<Termbox> {
    unsafe {
      if _is_open.swap(true, Ordering::Acquire) {
        return Err(Error::TerminalLocked);
      }
      match ffi::tb_init() {
        0 => {
          return Ok(Termbox{uninstantiable:()});
        },
        ffi::TB_EUNSUPPORTED_TERMINAL => {
          _is_open.store(false, Ordering::Release);
          return Err(Error::UnsupportedTerminal);
        },
        ffi::TB_EFAILED_TO_OPEN_TTY => {
          _is_open.store(false, Ordering::Release);
          return Err(Error::FailedToOpenTty);
        },
        ffi::TB_EPIPE_TRAP_ERROR => {
          _is_open.store(false, Ordering::Release);
          return Err(Error::PipeTrapError);
        },
        _ => {
          _is_open.store(false, Ordering::Release);
          return Err(Error::UnknownInitializationFailure);
        },
      }
    }
  }

  pub fn peek_event (&mut self, timeout: u32) -> Result<Event> {
    unsafe {
      let mut event = zeroed();
      let result = ffi::tb_peek_event(&mut event, timeout as c_int);
      if result < 0 {
        return Err(Error::PollFailed);
      } else if result == 0 {
        return Err(Error::Timeout);
      } else {
        match event.to_safe_event() {
          Some(event) => { return Ok(event); },
          None => { return Err(Error::UnknownEvent); },
        }
      }
    }
  }

  pub fn poll_event (&mut self) -> Result<Event> {
    unsafe {
      let mut event = zeroed();
      let result = ffi::tb_poll_event(&mut event);
      if result <= 0 {
        return Err(Error::PollFailed);
      } else {
        match event.to_safe_event() {
          Some(event) => { return Ok(event); },
          None => { return Err(Error::UnknownEvent) },
        }
      }
    }
  }

  pub fn present (&mut self) {
    unsafe {
      ffi::tb_present();
    }
  }

  pub fn put_cell (&mut self, x: i32, y: i32, cell: Cell) {
    unsafe {
      ffi::tb_put_cell(x as c_int, y as c_int, &cell);
    }
  }

  pub fn set_clear_attributes (&mut self, fg: u16, bg: u16) {
    unsafe {
      ffi::tb_set_clear_attributes(fg, bg);
    }
  }

  pub fn set_cursor (&mut self, x: i32, y: i32) {
    unsafe {
      ffi::tb_set_cursor(x as c_int, y as c_int);
    }
  }

  pub fn set_input_mode (&mut self, mode: InputMode) {
    unsafe {
      ffi::tb_select_input_mode(mode.to_c_int());
    }
  }

  pub fn set_output_mode (&mut self, mode: OutputMode) {
    unsafe {
      ffi::tb_select_output_mode(mode.to_c_int());
    }
  }
}

impl Drop for Termbox {
  fn drop (&mut self) {
    unsafe {
      ffi::tb_shutdown();
      _is_open.store(false, Ordering::Release);
    }
  }
}


//
// constants
//


// attributes
pub const DEFAULT: u16 = 0x00;
pub const BLACK: u16 = 0x01;
pub const RED: u16 = 0x02;
pub const GREEN: u16 = 0x03;
pub const YELLOW: u16 = 0x04;
pub const BLUE: u16 = 0x05;
pub const MAGENTA: u16 = 0x06;
pub const CYAN: u16 = 0x07;
pub const WHITE: u16 = 0x08;

pub const BOLD: u16 = 0x0100;
pub const UNDERLINE: u16 = 0x0200;
pub const REVERSE: u16 = 0x0400;

// keys
pub const KEY_CTRL_TILDE: u16 = 0x00;
pub const KEY_CTRL_2: u16 = 0x00;
pub const KEY_CTRL_A: u16 = 0x01;
pub const KEY_CTRL_B: u16 = 0x02;
pub const KEY_CTRL_C: u16 = 0x03;
pub const KEY_CTRL_D: u16 = 0x04;
pub const KEY_CTRL_E: u16 = 0x05;
pub const KEY_CTRL_F: u16 = 0x06;
pub const KEY_CTRL_G: u16 = 0x07;
pub const KEY_BACKSPACE: u16 = 0x08;
pub const KEY_CTRL_H: u16 = 0x08;
pub const KEY_TAB: u16 = 0x09;
pub const KEY_CTRL_I: u16 = 0x09;
pub const KEY_CTRL_J: u16 = 0x0a;
pub const KEY_CTRL_K: u16 = 0x0b;
pub const KEY_CTRL_L: u16 = 0x0c;
pub const KEY_ENTER: u16 = 0x0d;
pub const KEY_CTRL_M: u16 = 0x0d;
pub const KEY_CTRL_N: u16 = 0x0e;
pub const KEY_CTRL_O: u16 = 0x0f;
pub const KEY_CTRL_P: u16 = 0x10;
pub const KEY_CTRL_Q: u16 = 0x11;
pub const KEY_CTRL_R: u16 = 0x12;
pub const KEY_CTRL_S: u16 = 0x13;
pub const KEY_CTRL_T: u16 = 0x14;
pub const KEY_CTRL_U: u16 = 0x15;
pub const KEY_CTRL_V: u16 = 0x16;
pub const KEY_CTRL_W: u16 = 0x17;
pub const KEY_CTRL_X: u16 = 0x18;
pub const KEY_CTRL_Y: u16 = 0x19;
pub const KEY_CTRL_Z: u16 = 0x1a;
pub const KEY_ESC: u16 = 0x1b;
pub const KEY_CTRL_LSQ_BRACKET: u16 = 0x1b;
pub const KEY_CTRL_3: u16 = 0x1b;
pub const KEY_CTRL_4: u16 = 0x1c;
pub const KEY_CTRL_BACKSLASH: u16 = 0x1c;
pub const KEY_CTRL_5: u16 = 0x1d;
pub const KEY_CTRL_RSQ_BRACKET: u16 = 0x1d;
pub const KEY_CTRL_6: u16 = 0x1e;
pub const KEY_CTRL_7: u16 = 0x1f;
pub const KEY_CTRL_SLASH: u16 = 0x1f;
pub const KEY_CTRL_UNDERSCORE: u16 = 0x1f;
pub const KEY_SPACE: u16 = 0x20;
pub const KEY_BACKSPACE2: u16 = 0x7f;
pub const KEY_CTRL_8: u16 = 0x7f;

pub const KEY_F1: u16 = 0xffff - 0;
pub const KEY_F2: u16 = 0xffff - 1;
pub const KEY_F3: u16 = 0xffff - 2;
pub const KEY_F4: u16 = 0xffff - 3;
pub const KEY_F5: u16 = 0xffff - 4;
pub const KEY_F6: u16 = 0xffff - 5;
pub const KEY_F7: u16 = 0xffff - 6;
pub const KEY_F8: u16 = 0xffff - 7;
pub const KEY_F9: u16 = 0xffff - 8;
pub const KEY_F10: u16 = 0xffff - 9;
pub const KEY_F11: u16 = 0xffff - 10;
pub const KEY_F12: u16 = 0xffff - 11;
pub const KEY_INSERT: u16 = 0xffff - 12;
pub const KEY_DELETE: u16 = 0xffff - 13;
pub const KEY_HOME: u16 = 0xffff - 14;
pub const KEY_END: u16 = 0xffff - 15;
pub const KEY_PGUP: u16 = 0xffff - 16;
pub const KEY_PGDN: u16 = 0xffff - 17;
pub const KEY_ARROW_UP: u16 = 0xffff - 18;
pub const KEY_ARROW_DOWN: u16 = 0xffff - 19;
pub const KEY_ARROW_LEFT: u16 = 0xffff - 20;
pub const KEY_ARROW_RIGHT: u16 = 0xffff - 21;
