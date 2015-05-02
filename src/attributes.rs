// Copyright (c) 2015, <daggerbot@gmail.com>
// This software is available under the terms of the zlib license.
// See COPYING.TXT for more information.

//! All constants defined here are valid only for `OutputMode::Normal`.

/// Determines the appearance of a character cell.
/// Each cell has a foreground attribute and a background attribute.
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

/// Use a lighter variation of one of the standard colors.
pub const BOLD: Attribute = ::ffi::TB_BOLD;
/// Put an underline under the displayed character if the terminal supports it.
pub const UNDERLINE: Attribute = ::ffi::TB_UNDERLINE;
pub const REVERSE: Attribute = ::ffi::TB_REVERSE;
