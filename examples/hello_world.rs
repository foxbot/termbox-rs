// Copyright (c) 2015, <daggerbot@gmail.com>
// This software is available under the terms of the zlib license.
// See COPYING.TXT for more information.

extern crate termbox;

use termbox::{
  Termbox,
  Event,
  DEFAULT,
  KEY_ESC,
};

fn main () {
  // Open the terminal
  let mut tb = Termbox::open().unwrap();

  // Clear the screen to black
  tb.set_clear_attributes(DEFAULT, DEFAULT);
  tb.clear();

  // Display a message
  tb.put_str(0, 0, "Hello, world!", DEFAULT, DEFAULT);
  tb.put_str(0, 1, "Press Esc to continue", DEFAULT, DEFAULT);
  tb.present();

  // Wait for the user to press Esc
  loop {
    match tb.poll_event() {
      Event::Key(event) => {
        if event.key == KEY_ESC {
          break;
        }
      },
      _ => {},
    }
  }
}
