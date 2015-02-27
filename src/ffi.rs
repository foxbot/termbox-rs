// Rust language wrapper for Termbox.
// Termbox was written in C by nsf <no.smile.face@gmail.com>.
// Project page: https://code.google.com/p/termbox/
// This binding is subject to the terms of the original library.

use std::char::from_u32;

use libc::c_int;

use ::{
  Cell,
  Event,
  Mods,
  KeySym,
};

// event kinds
pub const TB_EVENT_KEY: u8 = 1;
pub const TB_EVENT_RESIZE: u8 = 2;

// event struct
#[derive(Copy)]
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
      TB_EVENT_KEY => Some(Event::Key(KeySym {
        mods: match Mods::from_bits(self.mods) { Some(mods) => mods, None => Mods::empty() },
        key: self.key,
        ch: match from_u32(self.ch) { Some(ch) => ch, None => 0 as char },
      })),
      TB_EVENT_RESIZE => Some(Event::Resize(self.w, self.h)),
      _ => None
    }
  }
}

// init results
pub const TB_EUNSUPPORTED_TERMINAL: c_int = -1;
pub const TB_EFAILED_TO_OPEN_TTY: c_int = -2;
pub const TB_EPIPE_TRAP_ERROR: c_int = -3;

// input modes
pub const TB_INPUT_CURRENT: c_int = 0;
pub const TB_INPUT_ESC: c_int = 1;
pub const TB_INPUT_ALT: c_int = 2;

// output modes
pub const TB_OUTPUT_CURRENT: c_int = 0;
pub const TB_OUTPUT_NORMAL: c_int = 1;
pub const TB_OUTPUT_256: c_int = 2;
pub const TB_OUTPUT_216: c_int = 3;
pub const TB_OUTPUT_GRAYSCALE: c_int = 4;

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