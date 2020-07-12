use crate::{kernel, print};
use alloc::string::String;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts;

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
    if let Some(fg) = color_to_fg(name) {
        Some(fg + 10)
    } else {
        None
    }
}

lazy_static! {
    pub static ref STDIN: Mutex<String> = Mutex::new(String::new());
    pub static ref ECHO: Mutex<bool> = Mutex::new(true);
    pub static ref RAW: Mutex<bool> = Mutex::new(false);
}

pub fn has_cursor() -> bool {
    cfg!(feature = "vga")
}

pub fn clear_row_after(x: usize) {
    if cfg!(feature = "vga") {
        kernel::vga::clear_row_after(x);
    } else {
        print!("\r"); // Move cursor to begining of line
        print!("\x1b[{}C", x); // Move cursor forward to position
        print!("\x1b[K"); // Clear line after position
    }
}

pub fn cursor_position() -> (usize, usize) {
    if cfg!(feature = "vga") {
        kernel::vga::cursor_position()
    } else {
        print!("\x1b[6n"); // Ask cursor position
        get_char(); // ESC
        get_char(); // [
        let mut x = String::new();
        let mut y = String::new();
        loop {
            let c = get_char();
            if c == ';' {
                break;
            } else {
                y.push(c);
            }
        }
        loop {
            let c = get_char();
            if c == 'R' {
                break;
            } else {
                x.push(c);
            }
        }
        (x.parse().unwrap_or(1), y.parse().unwrap_or(1))
    }
}

pub fn set_writer_position(x: usize, y: usize) {
    if cfg!(feature = "vga") {
        kernel::vga::set_writer_position(x, y);
    } else {
        print!("\x1b[{};{}H", y + 1, x + 1);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        if cfg!(feature="vga") {
            $crate::kernel::vga::print_fmt(format_args!($($arg)*));
        } else {
            $crate::kernel::serial::print_fmt(format_args!($($arg)*));
        }
    });
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ({
        let uptime = $crate::kernel::clock::uptime();
        let csi_color = $crate::kernel::console::Style::color("LightGreen");
        let csi_reset = $crate::kernel::console::Style::reset();
        if cfg!(feature="vga") {
            $crate::kernel::vga::print_fmt(format_args!("{}[{:.6}]{} ", csi_color, uptime, csi_reset));
            $crate::kernel::vga::print_fmt(format_args!($($arg)*));
        } else {
            $crate::kernel::serial::print_fmt(format_args!("{}[{:.6}]{} ", csi_color, uptime, csi_reset));
            $crate::kernel::serial::print_fmt(format_args!($($arg)*));
        }
    });
}

pub fn disable_echo() {
    let mut echo = ECHO.lock();
    *echo = false;
}

pub fn enable_echo() {
    let mut echo = ECHO.lock();
    *echo = true;
}

pub fn is_echo_enabled() -> bool {
    *ECHO.lock()
}

pub fn disable_raw() {
    let mut raw = RAW.lock();
    *raw = false;
}

pub fn enable_raw() {
    let mut raw = RAW.lock();
    *raw = true;
}

pub fn is_raw_enabled() -> bool {
    *RAW.lock()
}

pub fn key_handle(key: char) {
    let mut stdin = STDIN.lock();

    if key == '\x08' && !is_raw_enabled() {
        // Avoid printing more backspaces than chars inserted into STDIN.
        // Also, the VGA driver support only ASCII so unicode chars will
        // be displayed with one square for each codepoint.
        if stdin.len() > 0 {
            let n = stdin.pop().unwrap().len_utf8();
            if is_echo_enabled() {
                for _ in 0..n {
                    print!("\x08");
                }
            }
        }
    } else {
        // TODO: Replace non-ascii chars by ascii square symbol to keep length
        // at 1 instead of being variable?
        stdin.push(key);
        if is_echo_enabled() {
            print!("{}", key);
        }
    }
}

pub fn get_char() -> char {
    kernel::console::disable_echo();
    kernel::console::enable_raw();
    loop {
        kernel::time::halt();
        let res = interrupts::without_interrupts(|| {
            let mut stdin = STDIN.lock();
            match stdin.chars().next_back() {
                Some(c) => {
                    stdin.clear();
                    Some(c)
                },
                _ => {
                    None
                }
            }
        });
        if let Some(c) = res {
            kernel::console::enable_echo();
            kernel::console::disable_raw();
            return c;
        }
    }
}

pub fn get_line() -> String {
    loop {
        kernel::time::halt();
        let res = interrupts::without_interrupts(|| {
            let mut stdin = STDIN.lock();
            match stdin.chars().next_back() {
                Some('\n') => {
                    let line = stdin.clone();
                    stdin.clear();
                    Some(line)
                }
                _ => {
                    None
                }
            }
        });
        if let Some(line) = res {
            return line;
        }
    }
}
