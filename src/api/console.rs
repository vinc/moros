use crate::sys;
use core::fmt;

pub use crate::sys::console::{ETX_KEY, EOT_KEY};

#[derive(Clone, Copy)]
pub struct Style {
    foreground: Option<usize>,
    background: Option<usize>,
}

impl Style {
    pub fn reset() -> Self {
        Self { foreground: None, background: None }
    }

    pub fn foreground(name: &str) -> Self {
        Self { foreground: color_to_fg(name), background: None }
    }

    pub fn with_foreground(self, name: &str) -> Self {
        Self { foreground: color_to_fg(name), background: self.background }
    }

    pub fn background(name: &str) -> Self {
        Self { foreground: None, background: color_to_bg(name) }
    }

    pub fn with_background(self, name: &str) -> Self {
        Self { foreground: self.foreground, background: color_to_bg(name) }
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
        "Black"      => Some(30),
        "Red"        => Some(31),
        "Green"      => Some(32),
        "Brown"      => Some(33),
        "Blue"       => Some(34),
        "Magenta"    => Some(35),
        "Cyan"       => Some(36),
        "LightGray"  => Some(37),
        "DarkGray"   => Some(90),
        "LightRed"   => Some(91),
        "LightGreen" => Some(92),
        "Yellow"     => Some(93),
        "LightBlue"  => Some(94),
        "Pink"       => Some(95),
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
