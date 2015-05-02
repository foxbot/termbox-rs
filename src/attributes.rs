// Copyright (c) 2015, <daggerbot@gmail.com>
// This software is available under the terms of the zlib license.
// See COPYING.TXT for more information.

pub type Attribute = u16;

pub const DEFAULT: Attribute = ::ffi::TB_DEFAULT;
pub const BLACK: Attribute = ::ffi::TB_BLACK;
pub const RED: Attribute = ::ffi::TB_RED;
pub const GREEN: Attribute = ::ffi::TB_GREEN;
pub const YELLOW: Attribute = ::ffi::TB_YELLOW;
pub const BLUE: Attribute = ::ffi::TB_BLUE;
pub const MAGENTA: Attribute = ::ffi::TB_MAGENTA;
pub const CYAN: Attribute = ::ffi::TB_CYAN;
pub const WHITE: Attribute = ::ffi::TB_WHITE;

pub const BOLD: Attribute = ::ffi::TB_BOLD;
pub const UNDERLINE: Attribute = ::ffi::TB_UNDERLINE;
pub const REVERSE: Attribute = ::ffi::TB_REVERSE;
