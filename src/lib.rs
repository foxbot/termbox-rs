// Copyright (c) 2015, <daggerbot@gmail.com>
// This software is available under the terms of the zlib license.
// See COPYING.TXT for more information.

#![crate_name="termbox"]
#![crate_type="lib"]

extern crate termbox_sys as ffi;
extern crate libc;
extern crate num;

mod internal;

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

pub type Attribute = u16;
pub type Cell = ffi::RawCell;
pub type Coord = c_int;
pub type Key = u16;
pub type Time = c_int;


//
// Event
//


#[derive(Clone, Copy, Debug)]
pub enum Event {
  Key(KeyEvent),
  Resize(ResizeEvent),
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
  Esc,
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
  pub key: Key,
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
  Normal,
  Color256,
  Color216,
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


pub struct Termbox {
  #[allow(dead_code)]
  lock: Lock,
}

impl Termbox {
  pub fn blit (&mut self, x: Coord, y: Coord, w: Coord, h: Coord, cells: &[Cell]) {
    unsafe {
      let uwidth: usize = NumCast::from(w).unwrap();
      let uheight: usize = NumCast::from(h).unwrap();
      let min_len = CheckedMul::checked_mul(&uwidth, &uheight).unwrap();
      assert!(cells.len() >= min_len);
      ffi::tb_blit(x, y, w, h, &cells[0]);
    }
  }

  pub fn cell_buffer<'a> (&'a self) -> &'a [Cell] {
    unsafe {
      let w: usize = NumCast::from(ffi::tb_width()).unwrap();
      let h: usize = NumCast::from(ffi::tb_height()).unwrap();
      let len = CheckedMul::checked_mul(&w, &h).unwrap();
      let ptr = ffi::tb_cell_buffer() as *const Cell;
      return from_raw_parts(ptr, len);
    }
  }

  pub fn cell_buffer_mut<'a> (&'a mut self) -> &'a mut [Cell] {
    unsafe {
      let w: usize = NumCast::from(ffi::tb_width()).unwrap();
      let h: usize = NumCast::from(ffi::tb_height()).unwrap();
      let len = CheckedMul::checked_mul(&w, &h).unwrap();
      let ptr = ffi::tb_cell_buffer();
      return from_raw_parts_mut(ptr, len);
    }
  }

  pub fn change_cell (&mut self, x: Coord, y: Coord, ch: char, fg: Attribute, bg: Attribute) {
    unsafe {
      ffi::tb_change_cell(x, y, ch as u32, fg, bg);
    }
  }

  pub fn clear (&mut self) {
    unsafe {
      ffi::tb_clear();
    }
  }

  pub fn height (&self) -> Coord {
    unsafe {
      ffi::tb_height()
    }
  }

  pub fn hide_cursor (&mut self) {
    unsafe {
      ffi::tb_set_cursor(ffi::TB_HIDE_CURSOR, ffi::TB_HIDE_CURSOR);
    }
  }

  pub fn input_mode (&self) -> InputMode {
    unsafe {
      let raw_mode = ffi::tb_select_input_mode(ffi::TB_INPUT_CURRENT);
      return InputMode::from_raw(raw_mode).unwrap();
    }
  }

  pub fn is_mouse_enabled (&self) -> bool {
    unsafe {
      let mode = ffi::tb_select_input_mode(ffi::TB_INPUT_CURRENT);
      return (mode & ffi::TB_INPUT_MOUSE) != 0;
    }
  }

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

  pub fn output_mode (&self) -> OutputMode {
    unsafe {
      let raw_mode = ffi::tb_select_output_mode(ffi::TB_OUTPUT_CURRENT);
      return OutputMode::from_raw(raw_mode).unwrap();
    }
  }

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

  pub fn present (&mut self) {
    unsafe {
      ffi::tb_present();
    }
  }

  pub fn put_cell (&mut self, x: Coord, y: Coord, cell: Cell) {
    unsafe {
      ffi::tb_put_cell(x, y, &cell);
    }
  }

  pub fn put_str (&mut self, x: Coord, y: Coord, msg: &str, fg: Attribute, bg: Attribute) {
    unsafe {
      let mut x = x;
      for ch in msg.chars() {
        ffi::tb_change_cell(x, y, ch as u32, fg, bg);
        x += 1;
      }
    }
  }

  pub fn set_clear_attributes (&mut self, fg: Attribute, bg: Attribute) {
    unsafe {
      ffi::tb_set_clear_attributes(fg, bg);
    }
  }

  pub fn set_cursor (&mut self, x: Coord, y: Coord) {
    unsafe {
      ffi::tb_set_cursor(x, y);
    }
  }

  pub fn set_input_mode (&mut self, mode: InputMode) {
    unsafe {
      let prev_mode = ffi::tb_select_input_mode(ffi::TB_INPUT_CURRENT);
      let flags = prev_mode & !INPUT_MODE_MASK;
      ffi::tb_select_input_mode(mode.to_raw() | flags);
    }
  }

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

  pub fn set_output_mode (&mut self, mode: OutputMode) {
    unsafe {
      ffi::tb_select_output_mode(mode.to_raw());
    }
  }

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
