// Rust language wrapper for Termbox.
// Termbox was written in C by nsf <no.smile.face@gmail.com>.
// Project page: https://code.google.com/p/termbox/
// This binding is subject to the terms of the original library.

#![crate_name="termbox"]
#![crate_type="lib"]

#![feature(struct_variant)]

extern crate libc;

use std::default::Default;
use std::sync::atomics::{
  AtomicBool,
  SeqCst,
  INIT_ATOMIC_BOOL,
};
use libc::c_int;

// globals :((((((
static mut _is_open: AtomicBool = INIT_ATOMIC_BOOL;

// TODO: When rust 0.13 is released, add get_cell_buffer and get_cell_buffer_mut.


//
// Cell
//


#[repr(C)]
pub struct Cell {
  pub ch: char,
  pub fg: u16,
  pub bg: u16,
}

impl Default for Cell {
  fn default () -> Cell {
    Cell {
      ch: 0 as char,
      fg: 0,
      bg: 0,
    }
  }
}


//
// Event
//


pub enum Event {
  KeyEvent {
    pub mods: Mods,
    pub key: u16,
    pub ch: char,
  },
  ResizeEvent {
    pub w: i32,
    pub h: i32,
  },
}


//
// InputMode
//


#[deriving(Eq, PartialEq)]
pub enum InputMode {
  InputEsc,
  InputAlt,
}

impl InputMode {
  pub fn from_c_int (mode: c_int) -> Option<InputMode> {
    match mode {
      ffi::TB_INPUT_ESC => Some(InputEsc),
      ffi::TB_INPUT_ALT => Some(InputAlt),
      _ => None
    }
  }

  pub fn to_c_int (self) -> c_int {
    match self {
      InputEsc => ffi::TB_INPUT_ESC,
      InputAlt => ffi::TB_INPUT_ALT,
    }
  }
}


//
// Mods
//


bitflags! {
  flags Mods: u8 {
    static MOD_ALT = 0x01,
  }
}


//
// OutputMode
//


#[deriving(Eq, PartialEq)]
pub enum OutputMode {
  OutputNormal,
  Output256,
  Output216,
  OutputGrayscale,
}

impl OutputMode {
  pub fn from_c_int (mode: c_int) -> Option<OutputMode> {
    match mode {
      ffi::TB_OUTPUT_NORMAL => Some(OutputNormal),
      ffi::TB_OUTPUT_256 => Some(Output256),
      ffi::TB_OUTPUT_216 => Some(Output216),
      ffi::TB_OUTPUT_GRAYSCALE => Some(OutputGrayscale),
      _ => None
    }
  }

  pub fn to_c_int (self) -> c_int {
    match self {
      OutputNormal => ffi::TB_OUTPUT_NORMAL,
      Output256 => ffi::TB_OUTPUT_256,
      Output216 => ffi::TB_OUTPUT_216,
      OutputGrayscale => ffi::TB_OUTPUT_GRAYSCALE,
    }
  }
}


//
// Termbox
//


pub struct Termbox {
  is_open: bool,
}

impl Termbox {
  pub fn blit (&mut self, x: i32, y: i32, w: i32, h: i32, cells: &[Cell]) {
    unsafe {
      if w > 0 && h > 0 {
        assert!(cells.len() >= (w * h) as uint);
      }
      if !self.is_open {
        return;
      }
      // check dimensions
      let (buffer_width, buffer_height) = self.get_size();
      if w < 1 || h < 1 || x + w <= 0 || y + h <= 0 || x >= buffer_width || y >= buffer_height {
        return;
      }
      // get valid bounds
      let min_x = if x < 0 {-x} else {0};
      let min_y = if y < 0 {-y} else {0};
      let max_x = std::cmp::min(x + w, buffer_width) - x;
      let max_y = std::cmp::min(y + h, buffer_height) - y;
      // blit
      std::slice::raw::mut_buf_as_slice(ffi::tb_cell_buffer(), (buffer_width * buffer_height) as uint, |dst| {
        for cy in std::iter::range(min_y, max_y) {
          let mut src_index = (cy * w + min_x) as uint;
          let mut dst_index = ((y + cy) * buffer_width + x + min_x) as uint;
          for _ in std::iter::range(min_x, max_x) {
            dst[dst_index] = cells[src_index];
            src_index += 1;
            dst_index += 1;
          }
        }
      });
    }
  }

  pub fn change_cell (&mut self, x: i32, y: i32, ch: char, fg: u16, bg: u16) {
    unsafe {
      if self.is_open {
        ffi::tb_change_cell(x as c_int, y as c_int, ch as u32, fg, bg);
      }
    }
  }

  pub fn clear (&mut self) {
    unsafe {
      if self.is_open {
        ffi::tb_clear();
      }
    }
  }

  pub fn close (&mut self) {
    unsafe {
      if self.is_open {
        self.is_open = false;
        ffi::tb_shutdown();
        _is_open.store(false, SeqCst);
      }
    }
  }

  pub fn get_input_mode (&self) -> Option<InputMode> {
    unsafe {
      if self.is_open {
        let mode = InputMode::from_c_int(ffi::tb_select_input_mode(ffi::TB_INPUT_CURRENT));
        assert!(mode != None);
        return mode;
      } else {
        return None;
      }
    }
  }

  pub fn get_output_mode (&self) -> Option<OutputMode> {
    unsafe {
      if self.is_open {
        let mode = OutputMode::from_c_int(ffi::tb_select_output_mode(ffi::TB_OUTPUT_CURRENT));
        assert!(mode != None);
        return mode;
      } else {
        return None;
      }
    }
  }

  pub fn get_size (&self) -> (i32, i32) {
    unsafe {
      if self.is_open {
        (ffi::tb_width() as i32, ffi::tb_height() as i32)
      } else {
        (0, 0)
      }
    }
  }

  pub fn hide_cursor (&mut self) {
    self.set_cursor(-1, -1);
  }

  pub fn is_open (&self) -> bool {
    self.is_open
  }

  pub fn new () -> Result<Termbox, String> {
    unsafe {
      let was_open = _is_open.swap(true, SeqCst);
      if was_open {
        return Err(String::from_str("only one instance of Termbox is allowed"));
      }
      match ffi::tb_init() {
        0 => {
          return Ok(Termbox{is_open: true});
        },
        ffi::TB_EUNSUPPORTED_TERMINAL => {
          _is_open.store(false, SeqCst);
          return Err(String::from_str("unsupported terminal"));
        },
        ffi::TB_EFAILED_TO_OPEN_TTY => {
          _is_open.store(false, SeqCst);
          return Err(String::from_str("failed to open tty"));
        },
        ffi::TB_EPIPE_TRAP_ERROR => {
          _is_open.store(false, SeqCst);
          return Err(String::from_str("pipe trap error"));
        },
        result => {
          _is_open.store(false, SeqCst);
          return Err(format!("tb_init returned {}", result));
        }
      }
    }
  }

  pub fn peek_event (&mut self, timeout: u32) -> Option<Event> {
    unsafe {
      if self.is_open {
        let mut event = Default::default();
        let result = ffi::tb_peek_event(&mut event, timeout as c_int);
        if result < 0 {
          fail!("tb_peek_event returned {}", result);
        } else if result == 0 {
          return None;
        } else {
          match event.to_safe_event() {
            Some(event) => {
              return Some(event);
            },
            None => {
              fail!("invalid event");
            },
          }
        }
      } else {
        return None;
      }
    }
  }

  pub fn poll_event (&mut self) -> Event {
    unsafe {
      if self.is_open {
        let mut event = Default::default();
        let result = ffi::tb_poll_event(&mut event);
        if result <= 0 {
          fail!("tb_poll_event returned {}", result);
        } else {
          match event.to_safe_event() {
            Some(event) => {
              return event;
            },
            None => {
              fail!("invalid event");
            },
          }
        }
      } else {
        fail!("Termbox is closed");
      }
    }
  }

  pub fn present (&mut self) {
    unsafe {
      if self.is_open {
        ffi::tb_present();
      }
    }
  }

  pub fn put_cell (&mut self, x: i32, y: i32, cell: Cell) {
    unsafe {
      if self.is_open {
        ffi::tb_put_cell(x as c_int, y as c_int, &cell);
      }
    }
  }

  pub fn set_clear_attributes (&mut self, fg: u16, bg: u16) {
    unsafe {
      if self.is_open {
        ffi::tb_set_clear_attributes(fg, bg);
      }
    }
  }

  pub fn set_cursor (&mut self, x: i32, y: i32) {
    unsafe {
      if self.is_open {
        ffi::tb_set_cursor(x as c_int, y as c_int);
      }
    }
  }

  pub fn set_input_mode (&mut self, mode: InputMode) {
    unsafe {
      if self.is_open {
        let imode = mode.to_c_int();
        let result = ffi::tb_select_input_mode(imode);
        assert!(result == imode);
      }
    }
  }

  pub fn set_output_mode (&mut self, mode: OutputMode) {
    unsafe {
      if self.is_open {
        let imode = mode.to_c_int();
        let result = ffi::tb_select_output_mode(imode);
        assert!(result == imode);
      }
    }
  }
}

impl Drop for Termbox {
  fn drop (&mut self) {
    self.close();
  }
}


//
// ffi
//


#[allow(non_camel_case_types)]
pub mod ffi {
  use std;
  use std::default::Default;
  use libc::c_int;

  use super::{
    Cell,
    Event,
    Mods,
    KeyEvent,
    ResizeEvent,
  };

  // event kinds
  pub static TB_EVENT_KEY:    u8 = 1;
  pub static TB_EVENT_RESIZE: u8 = 2;

  // event struct
  #[repr(C)]
  pub struct tb_event {
    pub kind: u8,
    pub mods: u8,
    pub key: u16,
    pub ch: u32,
    pub w: i32,
    pub h: i32,
  }

  impl tb_event {
    pub fn to_safe_event (&self) -> Option<Event> {
      match self.kind {
        TB_EVENT_KEY => Some(KeyEvent {
          mods: match Mods::from_bits(self.mods) { Some(mods) => mods, None => Mods::empty() },
          key: self.key,
          ch: match std::char::from_u32(self.ch) { Some(ch) => ch, None => 0 as char },
        }),
        TB_EVENT_RESIZE => Some(ResizeEvent {
          w: self.w,
          h: self.h,
        }),
        _ => None
      }
    }
  }

  impl Default for tb_event {
    fn default () -> tb_event {
      tb_event {
        kind: 0,
        mods: 0,
        key: 0,
        ch: 0,
        w: 0,
        h: 0,
      }
    }
  }

  // init results
  pub static TB_EUNSUPPORTED_TERMINAL: c_int = -1;
  pub static TB_EFAILED_TO_OPEN_TTY:   c_int = -2;
  pub static TB_EPIPE_TRAP_ERROR:      c_int = -3;

  // input modes
  pub static TB_INPUT_CURRENT: c_int = 0;
  pub static TB_INPUT_ESC:     c_int = 1;
  pub static TB_INPUT_ALT:     c_int = 2;

  // output modes
  pub static TB_OUTPUT_CURRENT:   c_int = 0;
  pub static TB_OUTPUT_NORMAL:    c_int = 1;
  pub static TB_OUTPUT_256:       c_int = 2;
  pub static TB_OUTPUT_216:       c_int = 3;
  pub static TB_OUTPUT_GRAYSCALE: c_int = 4;

  // functions
  #[link(name="termbox")]
  extern "C" {
    pub fn tb_blit (x: c_int, y: c_int, w: c_int, h: c_int, cells: *const Cell);
    pub fn tb_cell_buffer () -> *mut Cell;
    pub fn tb_change_cell (x: c_int, y: c_int, ch: u32, fg: u16, bg: u16);
    pub fn tb_clear ();
    pub fn tb_height () -> c_int;
    pub fn tb_init () -> c_int;
    pub fn tb_peek_event (event: *mut tb_event, timeout: c_int) -> c_int;
    pub fn tb_poll_event (event: *mut tb_event) -> c_int;
    pub fn tb_present ();
    pub fn tb_put_cell (x: c_int, y: c_int, cell: *const Cell);
    pub fn tb_select_input_mode (mode: c_int) -> c_int;
    pub fn tb_select_output_mode (mode: c_int) -> c_int;
    pub fn tb_set_clear_attributes (fg: u16, bg: u16);
    pub fn tb_set_cursor (x: c_int, y: c_int);
    pub fn tb_shutdown ();
    pub fn tb_width () -> c_int;
  }
}


//
// constants
//


// attributes
pub static DEFAULT: u16 = 0x00;
pub static BLACK: u16 = 0x01;
pub static RED: u16 = 0x02;
pub static GREEN: u16 = 0x03;
pub static YELLOW: u16 = 0x04;
pub static BLUE: u16 = 0x05;
pub static MAGENTA: u16 = 0x06;
pub static CYAN: u16 = 0x07;
pub static WHITE: u16 = 0x08;

pub static BOLD: u16 = 0x0100;
pub static UNDERLINE: u16 = 0x0200;
pub static REVERSE: u16 = 0x0400;

// keys
pub static KEY_CTRL_TILDE: u16 = 0x00;
pub static KEY_CTRL_2: u16 = 0x00;
pub static KEY_CTRL_A: u16 = 0x01;
pub static KEY_CTRL_B: u16 = 0x02;
pub static KEY_CTRL_C: u16 = 0x03;
pub static KEY_CTRL_D: u16 = 0x04;
pub static KEY_CTRL_E: u16 = 0x05;
pub static KEY_CTRL_F: u16 = 0x06;
pub static KEY_CTRL_G: u16 = 0x07;
pub static KEY_BACKSPACE: u16 = 0x08;
pub static KEY_CTRL_H: u16 = 0x08;
pub static KEY_TAB: u16 = 0x09;
pub static KEY_CTRL_I: u16 = 0x09;
pub static KEY_CTRL_J: u16 = 0x0a;
pub static KEY_CTRL_K: u16 = 0x0b;
pub static KEY_CTRL_L: u16 = 0x0c;
pub static KEY_ENTER: u16 = 0x0d;
pub static KEY_CTRL_M: u16 = 0x0d;
pub static KEY_CTRL_N: u16 = 0x0e;
pub static KEY_CTRL_O: u16 = 0x0f;
pub static KEY_CTRL_P: u16 = 0x10;
pub static KEY_CTRL_Q: u16 = 0x11;
pub static KEY_CTRL_R: u16 = 0x12;
pub static KEY_CTRL_S: u16 = 0x13;
pub static KEY_CTRL_T: u16 = 0x14;
pub static KEY_CTRL_U: u16 = 0x15;
pub static KEY_CTRL_V: u16 = 0x16;
pub static KEY_CTRL_W: u16 = 0x17;
pub static KEY_CTRL_X: u16 = 0x18;
pub static KEY_CTRL_Y: u16 = 0x19;
pub static KEY_CTRL_Z: u16 = 0x1a;
pub static KEY_ESC: u16 = 0x1b;
pub static KEY_CTRL_LSQ_BRACKET: u16 = 0x1b;
pub static KEY_CTRL_3: u16 = 0x1b;
pub static KEY_CTRL_4: u16 = 0x1c;
pub static KEY_CTRL_BACKSLASH: u16 = 0x1c;
pub static KEY_CTRL_5: u16 = 0x1d;
pub static KEY_CTRL_RSQ_BRACKET: u16 = 0x1d;
pub static KEY_CTRL_6: u16 = 0x1e;
pub static KEY_CTRL_7: u16 = 0x1f;
pub static KEY_CTRL_SLASH: u16 = 0x1f;
pub static KEY_CTRL_UNDERSCORE: u16 = 0x1f;
pub static KEY_SPACE: u16 = 0x20;
pub static KEY_BACKSPACE2: u16 = 0x7f;
pub static KEY_CTRL_8: u16 = 0x7f;

pub static KEY_F1: u16 = 0xffff - 0;
pub static KEY_F2: u16 = 0xffff - 1;
pub static KEY_F3: u16 = 0xffff - 2;
pub static KEY_F4: u16 = 0xffff - 3;
pub static KEY_F5: u16 = 0xffff - 4;
pub static KEY_F6: u16 = 0xffff - 5;
pub static KEY_F7: u16 = 0xffff - 6;
pub static KEY_F8: u16 = 0xffff - 7;
pub static KEY_F9: u16 = 0xffff - 8;
pub static KEY_F10: u16 = 0xffff - 9;
pub static KEY_F11: u16 = 0xffff - 10;
pub static KEY_F12: u16 = 0xffff - 11;
pub static KEY_INSERT: u16 = 0xffff - 12;
pub static KEY_DELETE: u16 = 0xffff - 13;
pub static KEY_HOME: u16 = 0xffff - 14;
pub static KEY_END: u16 = 0xffff - 15;
pub static KEY_PGUP: u16 = 0xffff - 16;
pub static KEY_PGDN: u16 = 0xffff - 17;
pub static KEY_ARROW_UP: u16 = 0xffff - 18;
pub static KEY_ARROW_DOWN: u16 = 0xffff - 19;
pub static KEY_ARROW_LEFT: u16 = 0xffff - 20;
pub static KEY_ARROW_RIGHT: u16 = 0xffff - 21;
