// Copyright (c) 2015, <daggerbot@gmail.com>
// This software is available under the terms of the zlib license.
// See COPYING.TXT for more information.

use std::sync::atomic::{
  AtomicBool,
  Ordering,
  ATOMIC_BOOL_INIT,
};

// Only allow Termbox to be used from one thread.
static mut _locked: AtomicBool = ATOMIC_BOOL_INIT;


//
// Lock
//


pub struct Lock {
  _uninstantiable: (),
}

impl Lock {
  pub fn acquire () -> Option<Lock> {
    unsafe {
      if _locked.swap(true, Ordering::SeqCst) {
        None
      } else {
        Some(Lock { _uninstantiable: () })
      }
    }
  }
}

impl Drop for Lock {
  fn drop (&mut self) {
    unsafe {
      _locked.store(false, Ordering::SeqCst);
    }
  }
}
