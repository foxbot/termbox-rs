// Copyright (c) 2015, <daggerbot@gmail.com>
// This software is available under the terms of the zlib license.
// See COPYING.TXT for more information.

use std::sync::atomic::{
  AtomicBool,
  Ordering,
  ATOMIC_BOOL_INIT,
};

// Only allow Termbox to be used from one thread.
static mut LOCK_FLAG: AtomicBool = ATOMIC_BOOL_INIT;


//
// Lock
//


pub struct Lock {
  _uninstantiable: (),
}

impl Lock {
  pub fn acquire () -> Option<Lock> {
    unsafe {
      if LOCK_FLAG.swap(true, Ordering::Acquire) {
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
      LOCK_FLAG.store(false, Ordering::Release);
    }
  }
}
