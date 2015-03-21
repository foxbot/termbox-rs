// Rust language wrapper for Termbox.
// Termbox is available under the MIT license.
// These bindings are public domain.

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
