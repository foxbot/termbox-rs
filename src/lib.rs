// Copyright (c) 2015, <daggerbot@gmail.com>
// This software is available under the terms of the zlib license.
// See COPYING.TXT for more information.

//! Termbox is a simple library for writing text based user interfaces.
//! It was originally written in C, and this binding was not written by the original author.
//!
//! # Example
//!
//! ~~~
//! extern crate termbox;
//!
//! use termbox::{
//!   Termbox,
//!   Event,
//!   BLACK,
//!   WHITE,
//!   BOLD,
//!   KEY_ESC,
//! };
//!
//! fn main () {
//!   // Open the terminal
//!   let mut tb = Termbox::open().unwrap();
//!
//!   // Clear the screen to black
//!   tb.set_clear_attributes(BLACK, BLACK);
//!   tb.clear();
//!
//!   // Display a message
//!   tb.put_str(0, 0, "Hello, world!", WHITE | BOLD, BLACK);
//!   tb.put_str(0, 1, "Press Esc to continue", WHITE, BLACK);
//!   tb.present();
//!
//!   // Wait for the user to press Esc
//!   loop {
//!     match tb.poll_event() {
//!       Event::Key(event) => {
//!         if event.key == KEY_ESC {
//!           break;
//!         }
//!       },
//!       _ => {},
//!     }
//!   }
//! }
//! ~~~

#![crate_name="termbox"]
#![crate_type="lib"]

extern crate termbox_sys as ffi;
extern crate libc;
extern crate num;

/// Contains the `Attribute` type and attribute constants.
pub mod attributes;
/// Contains the `Key` type and key constants.
pub mod keys;

mod internal;

pub use self::attributes::*;
pub use self::keys::*;

use std::char::from_u32;
use std::error::Error;
use std::fmt::{
  Display,
  Formatter,
};
use std::mem::uninitialized;
use std::slice::{
  from_raw_parts,
  from_raw_parts_mut,
};

use libc::c_int;
use num::{
  CheckedMul,
  NumCast,
};

use internal::Lock;

/// Represents a single character cell in the terminal output.
///
/// ~~~
/// pub struct Cell {
///   // The character displayed by the cell. This is assumed to be a unicode character,
///   // but as termbox was written in C, this cannot be enforced at the language level.
///   pub ch: u32,
///   pub fg: Attribute,
///   pub bg: Attribute,
/// }
/// ~~~
pub type Cell = ffi::RawCell;

/// Integral type used to represent coordinates in cell space.
pub type Coord = c_int;

/// Integral type used to define a duration of time in milliseconds.
/// This is used by `Termbox::peek_event`.
pub type Time = c_int;


//
// Event
//


/// Represents an event that describes a user input action.
/// Events can be received with `Termbox::peek_event` or `Termbox::poll_event`.
#[derive(Clone, Copy, Debug)]
pub enum Event {
  /// Received when the user presses a key on the keyboard.
  Key(KeyEvent),
  /// Received when the user resizes the terminal window.
  Resize(ResizeEvent),
  /// Received when the user presses a mouse button or uses the mouse wheel on the terminal.
  /// Mouse events are disabled by default, and must be enabled with `Termbox::set_mouse_enabled`.
  Mouse(MouseEvent),
}

impl Event {
  fn from_raw (raw: ffi::RawEvent) -> Option<Event> {
    match raw.etype {
      ffi::TB_EVENT_KEY => Some(Event::Key(KeyEvent::from_raw(raw).unwrap())),
      ffi::TB_EVENT_RESIZE => Some(Event::Resize(ResizeEvent::from_raw(raw).unwrap())),
      ffi::TB_EVENT_MOUSE => Some(Event::Mouse(MouseEvent::from_raw(raw).unwrap())),
      _ => None,
    }
  }
}


//
// InitError
//


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InitError {
  Locked,
  UnsupportedTerminal,
  FailedToOpenTty,
  PipeTrapError,
}

impl InitError {
  pub fn as_str (self) -> &'static str {
    match self {
      InitError::Locked => "locked",
      InitError::UnsupportedTerminal => "unsupported terminal",
      InitError::FailedToOpenTty => "failed to open tty",
      InitError::PipeTrapError => "pipe trap error",
    }
  }
}

impl InitError {
  fn from_raw (raw: c_int) -> Option<InitError> {
    match raw {
      ffi::TB_EUNSUPPORTED_TERMINAL => Some(InitError::UnsupportedTerminal),
      ffi::TB_EFAILED_TO_OPEN_TTY => Some(InitError::FailedToOpenTty),
      ffi::TB_EPIPE_TRAP_ERROR => Some(InitError::PipeTrapError),
      _ => None,
    }
  }
}

impl Display for InitError {
  fn fmt (&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
    f.write_str(self.as_str())
  }
}

impl Error for InitError {
  fn description (&self) -> &str {
    self.as_str()
  }
}


//
// InputMode
//


// Not defined in termbox.
// Must cover all bits used by input modes, excluding flags such as TB_INPUT_MOUSE.
const INPUT_MODE_MASK: c_int = 3;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InputMode {
  /// An ESC sequence in the input buffer is interpreted as `KEY_ESC`.
  Esc,
  /// An ESC sequence in the input buffer is interpreted as the following key with the `alt`
  /// flag enabled.
  Alt,
}

impl InputMode {
  fn from_raw (raw: c_int) -> Option<InputMode> {
    match raw & INPUT_MODE_MASK {
      ffi::TB_INPUT_ESC => Some(InputMode::Esc),
      ffi::TB_INPUT_ALT => Some(InputMode::Alt),
      _ => None,
    }
  }

  fn to_raw (self) -> c_int {
    match self {
      InputMode::Esc => ffi::TB_INPUT_ESC,
      InputMode::Alt => ffi::TB_INPUT_ALT,
    }
  }
}


//
// KeyEvent
//


#[derive(Clone, Copy, Debug)]
pub struct KeyEvent {
  /// Code for the key that was pressed by the user. See the `keys` module.
  pub key: Key,
  /// If the pressed key can be translated into a Unicode character, this contains the code point.
  pub ch: Option<char>,
  pub alt: bool,
}

impl KeyEvent {
  fn from_raw (raw: ffi::RawEvent) -> Option<KeyEvent> {
    if raw.etype == ffi::TB_EVENT_KEY {
      Some(KeyEvent {
        key: raw.key,
        ch: from_u32(raw.ch),
        alt: (raw.emod & ffi::TB_MOD_ALT) != 0,
      })
    } else {
      None
    }
  }
}


//
// MouseButton
//


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MouseButton {
  Left,
  Right,
  Middle,
  Release,
  WheelUp,
  WheelDown,
}

impl MouseButton {
  fn from_raw (raw: u16) -> Option<MouseButton> {
    match raw {
      ffi::TB_KEY_MOUSE_LEFT => Some(MouseButton::Left),
      ffi::TB_KEY_MOUSE_RIGHT => Some(MouseButton::Right),
      ffi::TB_KEY_MOUSE_MIDDLE => Some(MouseButton::Middle),
      ffi::TB_KEY_MOUSE_RELEASE => Some(MouseButton::Release),
      ffi::TB_KEY_MOUSE_WHEEL_UP => Some(MouseButton::WheelUp),
      ffi::TB_KEY_MOUSE_WHEEL_DOWN => Some(MouseButton::WheelDown),
      _ => None,
    }
  }
}


//
// MouseEvent
//


/// Mouse events are disabled by default. Use `Termbox::set_mouse_enabled` to enable them.
#[derive(Clone, Copy, Debug)]
pub struct MouseEvent {
  pub button: MouseButton,
  pub x: Coord,
  pub y: Coord,
}

impl MouseEvent {
  fn from_raw (raw: ffi::RawEvent) -> Option<MouseEvent> {
    if raw.etype == ffi::TB_EVENT_MOUSE {
      Some(MouseEvent {
        button: MouseButton::from_raw(raw.key).unwrap(),
        x: NumCast::from(raw.x).unwrap(),
        y: NumCast::from(raw.y).unwrap(),
      })
    } else {
      None
    }
  }
}


//
// OutputMode
//


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OutputMode {
  /// Valid attributes are defined by constants found in the `attributes` module.
  Normal,
  /// 256 color mode.
  /// * `0x00 - 0x07`: The 8 standard colors defined in the `attributes` module.
  /// * `0x08 - 0x0f`: Bold variations of the 8 standard colors.
  /// * `0x10 - 0xe7`: 216 additional colors.
  /// * `0xe8 - 0xff`: 24 shades of gray.
  /// Due to the addition of the `DEFAULT` attribute, the 8 standard colors should have their value
  /// subtracted by one.
  Color256,
  /// Supports only the 216 colors from `0x10 - 0xe7` described above.
  Color216,
  /// Supports only the 24 shades of gray from `0xe8 - 0xff` described above.
  Grayscale,
}

impl OutputMode {
  fn from_raw (raw: c_int) -> Option<OutputMode> {
    match raw {
      ffi::TB_OUTPUT_NORMAL => Some(OutputMode::Normal),
      ffi::TB_OUTPUT_256 => Some(OutputMode::Color256),
      ffi::TB_OUTPUT_216 => Some(OutputMode::Color216),
      ffi::TB_OUTPUT_GRAYSCALE => Some(OutputMode::Grayscale),
      _ => None,
    }
  }

  fn to_raw (self) -> c_int {
    match self {
      OutputMode::Normal => ffi::TB_OUTPUT_NORMAL,
      OutputMode::Color256 => ffi::TB_OUTPUT_256,
      OutputMode::Color216 => ffi::TB_OUTPUT_216,
      OutputMode::Grayscale => ffi::TB_OUTPUT_GRAYSCALE,
    }
  }
}


//
// ResizeEvent
//


#[derive(Clone, Copy, Debug)]
pub struct ResizeEvent {
  pub w: Coord,
  pub h: Coord,
}

impl ResizeEvent {
  fn from_raw (raw: ffi::RawEvent) -> Option<ResizeEvent> {
    if raw.etype == ffi::TB_EVENT_RESIZE {
      Some(ResizeEvent {
        w: NumCast::from(raw.w).unwrap(),
        h: NumCast::from(raw.h).unwrap(),
      })
    } else {
      None
    }
  }
}


//
// Termbox
//


/// The main entry point for all termbox functions.
/// This ensures that the terminal can only be accessed from one thread.
/// Sadly, writing to `stdout` can potentially interfere with termbox output.
pub struct Termbox {
  #[allow(dead_code)]
  lock: Lock,
}

impl Termbox {
  /// Copies a rectangular region of cells from a slice to the output buffer.
  pub fn blit (&mut self, x: Coord, y: Coord, w: Coord, h: Coord, cells: &[Cell]) {
    unsafe {
      let uwidth: usize = NumCast::from(w).unwrap();
      let uheight: usize = NumCast::from(h).unwrap();
      let min_len = CheckedMul::checked_mul(&uwidth, &uheight).unwrap();
      assert!(cells.len() >= min_len);
      ffi::tb_blit(x, y, w, h, &cells[0]);
    }
  }

  /// Returns a slice representing the output buffer.
  pub fn cell_buffer<'a> (&'a self) -> &'a [Cell] {
    unsafe {
      let w: usize = NumCast::from(ffi::tb_width()).unwrap();
      let h: usize = NumCast::from(ffi::tb_height()).unwrap();
      let len = CheckedMul::checked_mul(&w, &h).unwrap();
      let ptr = ffi::tb_cell_buffer() as *const Cell;
      return from_raw_parts(ptr, len);
    }
  }

  /// Returns a mutable slice representing the output buffer.
  pub fn cell_buffer_mut<'a> (&'a mut self) -> &'a mut [Cell] {
    unsafe {
      let w: usize = NumCast::from(ffi::tb_width()).unwrap();
      let h: usize = NumCast::from(ffi::tb_height()).unwrap();
      let len = CheckedMul::checked_mul(&w, &h).unwrap();
      let ptr = ffi::tb_cell_buffer();
      return from_raw_parts_mut(ptr, len);
    }
  }

  /// Changes a single cell in the output buffer.
  pub fn change_cell (&mut self, x: Coord, y: Coord, ch: char, fg: Attribute, bg: Attribute) {
    unsafe {
      ffi::tb_change_cell(x, y, ch as u32, fg, bg);
    }
  }

  /// Clears the output buffer and sets all cell attributes to those specified with
  /// `set_clear_attributes`.
  pub fn clear (&mut self) {
    unsafe {
      ffi::tb_clear();
    }
  }

  /// Returns the height of the output buffer in character cells.
  pub fn height (&self) -> Coord {
    unsafe {
      ffi::tb_height()
    }
  }

  /// Sets the cursor to an invalid position, making it invisible to the user.
  pub fn hide_cursor (&mut self) {
    unsafe {
      ffi::tb_set_cursor(ffi::TB_HIDE_CURSOR, ffi::TB_HIDE_CURSOR);
    }
  }

  /// Returns the input mode. See `set_input_mode`.
  pub fn input_mode (&self) -> InputMode {
    unsafe {
      let raw_mode = ffi::tb_select_input_mode(ffi::TB_INPUT_CURRENT);
      return InputMode::from_raw(raw_mode).unwrap();
    }
  }

  /// Determines whether mouse events are enabled. See `set_mouse_enabled`.
  pub fn is_mouse_enabled (&self) -> bool {
    unsafe {
      let mode = ffi::tb_select_input_mode(ffi::TB_INPUT_CURRENT);
      return (mode & ffi::TB_INPUT_MOUSE) != 0;
    }
  }

  /// Locks the terminal to an instance of `Termbox`. Only one instance may exist in a process.
  pub fn open () -> Result<Termbox, InitError> {
    unsafe {
      let lock;
      match Lock::acquire() {
        Some(l) => { lock = l; },
        None => { return Err(InitError::Locked); },
      }

      match ffi::tb_init() {
        0 => { return Ok(Termbox { lock: lock }); },
        n => { return Err(InitError::from_raw(n).unwrap()); },
      }
    }
  }

  /// Returns the current output mode. See `set_output_mode`.
  pub fn output_mode (&self) -> OutputMode {
    unsafe {
      let raw_mode = ffi::tb_select_output_mode(ffi::TB_OUTPUT_CURRENT);
      return OutputMode::from_raw(raw_mode).unwrap();
    }
  }

  /// Waits up to `timeout` milliseconds for an event. If an event is received, that event is
  /// returned. Otherwise, `None` is returned. A `timeout` of zero can be specified to poll for
  /// events that have already been received without waiting.
  pub fn peek_event (&mut self, timeout: Time) -> Option<Event> {
    unsafe {
      let mut raw: ffi::RawEvent = uninitialized();
      let result = ffi::tb_peek_event(&mut raw, timeout);

      if result < 0 {
        panic!("tb_peek_event returned {}", result);
      } else if result == 0 {
        return None;
      } else {
        return Some(Event::from_raw(raw).unwrap());
      }
    }
  }

  /// Waits for an input event and returns it.
  pub fn poll_event (&mut self) -> Event {
    unsafe {
      let mut raw: ffi::RawEvent = uninitialized();
      let result = ffi::tb_poll_event(&mut raw);

      if result <= 0 {
        panic!("tb_poll_event returned {}", result);
      } else {
        return Event::from_raw(raw).unwrap();
      }
    }
  }

  /// Writes any changes to the output buffer into the terminal. This must be called in order for
  /// the user to see any changes.
  pub fn present (&mut self) {
    unsafe {
      ffi::tb_present();
    }
  }

  /// Changes a single character cell.
  pub fn put_cell (&mut self, x: Coord, y: Coord, cell: Cell) {
    unsafe {
      ffi::tb_put_cell(x, y, &cell);
    }
  }

  /// Writes a horizontal sequence of character cells without wrapping. This is just a quick and
  /// dirty way to write strings without providing many options.
  pub fn put_str (&mut self, x: Coord, y: Coord, msg: &str, fg: Attribute, bg: Attribute) {
    unsafe {
      let mut x = x;
      for ch in msg.chars() {
        ffi::tb_change_cell(x, y, ch as u32, fg, bg);
        x += 1;
      }
    }
  }

  /// Sets what attributes should be used when clearing the output buffer with `clear`.
  pub fn set_clear_attributes (&mut self, fg: Attribute, bg: Attribute) {
    unsafe {
      ffi::tb_set_clear_attributes(fg, bg);
    }
  }

  /// Sets the position of the cursor. If invalid coordinates are provided, the cursor is hidden.
  pub fn set_cursor (&mut self, x: Coord, y: Coord) {
    unsafe {
      ffi::tb_set_cursor(x, y);
    }
  }

  /// Sets the method termbox should use to handle ESC sequences in the input buffer.
  pub fn set_input_mode (&mut self, mode: InputMode) {
    unsafe {
      let prev_mode = ffi::tb_select_input_mode(ffi::TB_INPUT_CURRENT);
      let flags = prev_mode & !INPUT_MODE_MASK;
      ffi::tb_select_input_mode(mode.to_raw() | flags);
    }
  }

  /// Enables or disables mouse events. Mouse events are disabled by default.
  pub fn set_mouse_enabled (&mut self, enabled: bool) {
    unsafe {
      let prev_mode = ffi::tb_select_input_mode(ffi::TB_INPUT_CURRENT);
      let new_mode;

      if enabled {
        new_mode = prev_mode | ffi::TB_INPUT_MOUSE;
      } else {
        new_mode = prev_mode & !ffi::TB_INPUT_MOUSE;
      }

      if new_mode != prev_mode {
        ffi::tb_select_input_mode(new_mode);
      }
    }
  }

  /// Sets the method termbox should use to interpret output attributes.
  pub fn set_output_mode (&mut self, mode: OutputMode) {
    unsafe {
      ffi::tb_select_output_mode(mode.to_raw());
    }
  }

  /// Returns the width of the output buffer in character cells.
  pub fn width (&self) -> Coord {
    unsafe {
      ffi::tb_width()
    }
  }
}

impl Drop for Termbox {
  fn drop (&mut self) {
    unsafe {
      ffi::tb_shutdown();
    }
  }
}
