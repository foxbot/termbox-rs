// Rust language wrapper for Termbox.
// Termbox was written in C by nsf <no.smile.face@gmail.com>.
// Project page: https://code.google.com/p/termbox/
// This binding is subject to the terms of the original library.

use ::{
  Event,
  Termbox,
};


#[test]
fn test () {
  let mut termbox;
  match Termbox::new() {
    Ok(t) => { termbox = t; },
    Err(err) => { panic!("can't initialize termbox: {}", err); },
  }

  termbox.clear();
  termbox.put_str(0, 0, "Press any key...", 0, 0);
  termbox.present();

  loop {
    match termbox.poll_event() {
      Ok(event) => {
        match event {
          Event::Key(_) => { break; },
          _ => {},
        }
      }
      Err(err) => { panic!("poll failed: {}", err); },
    }
  }
}
