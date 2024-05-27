use crate::sys;

use alloc::string::ToString;
use core::fmt;

pub use crate::sys::console::{EOT_KEY, ETX_KEY};

#[derive(Clone, Copy)]
pub struct Style {
    foreground: Option<usize>,
    background: Option<usize>,
}

impl Style {
    pub fn reset() -> Self {
        Self {
            foreground: None,
            background: None,
        }
    }

    pub fn foreground(name: &str) -> Self {
        Self {
            foreground: color_to_fg(name),
            background: None,
        }
    }

    pub fn with_foreground(self, name: &str) -> Self {
        Self {
            foreground: color_to_fg(name),
            background: self.background,
        }
    }

    pub fn background(name: &str) -> Self {
        Self {
            foreground: None,
            background: color_to_bg(name),
        }
    }

    pub fn with_background(self, name: &str) -> Self {
        Self {
            foreground: self.foreground,
            background: color_to_bg(name),
        }
    }

    pub fn color(name: &str) -> Self {
        Self::foreground(name)
    }

    pub fn with_color(self, name: &str) -> Self {
        self.with_foreground(name)
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(fg) = self.foreground {
            if let Some(bg) = self.background {
                write!(f, "\x1b[{};{}m", fg, bg)
            } else {
                write!(f, "\x1b[{}m", fg)
            }
        } else if let Some(bg) = self.background {
            write!(f, "\x1b[{}m", bg)
        } else {
            write!(f, "\x1b[0m")
        }
    }
}

fn color_to_fg(name: &str) -> Option<usize> {
    match name {
        "black"      => Some(30),
        "maroon"        => Some(31),
        "green"      => Some(32),
        "olive"      => Some(33),
        "navy"       => Some(34),
        "purple"    => Some(35),
        "teal"       => Some(36),
        "LightGray"  => Some(37),
        "DarkGray"   => Some(90),
        "red"   => Some(91),
        "lime" => Some(92),
        "yellow"     => Some(93),
        "blue"  => Some(94),
        "fushia"       => Some(95),
        "LightCyan"  => Some(96),
        "White"      => Some(97),
        _            => None,
    }
}

fn color_to_bg(name: &str) -> Option<usize> {
    color_to_fg(name).map(|fg| fg + 10)
}

pub fn is_printable(c: char) -> bool {
    if cfg!(feature = "video") {
        // Check if the char can be converted to ASCII or Extended ASCII before
        // asking the VGA driver if it's printable.
        ((c as u32) < 0xFF) && sys::vga::is_printable(c as u8)
    } else {
        true // TODO
    }
}

// The size of the screen in VGA Text Mode is 80x25

pub fn cols() -> usize {
    let n = 80; // chars
    sys::process::env("COLS").unwrap_or(n.to_string()).parse().unwrap_or(n)
}

pub fn rows() -> usize {
    let n = 25; // lines
    sys::process::env("ROWS").unwrap_or(n.to_string()).parse().unwrap_or(n)
}
