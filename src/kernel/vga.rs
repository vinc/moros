use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::port::Port;

lazy_static! {
    /// A global `Writer` instance that can be used for printing to the VGA text buffer.
    ///
    /// Used by the `print!` and `println!` macros.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        col_pos: 0,
        row_pos: 0,
        color_code: ColorCode::new(Color::LightGray, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

/// The standard color palette in VGA text mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// A combination of a foreground and a background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    /// Create a new `ColorCode` with the given foreground and background colors.
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// A screen character in the VGA text buffer, consisting of an ASCII character and a `ColorCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// The height of the text buffer (normally 25 lines).
const BUFFER_HEIGHT: usize = 25;
/// The width of the text buffer (normally 80 columns).
const BUFFER_WIDTH: usize = 80;

/// A structure representing the VGA text buffer.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// A writer type that allows writing ASCII bytes and strings to an underlying `Buffer`.
///
/// Wraps lines at `BUFFER_WIDTH`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct Writer {
    col_pos: usize,
    row_pos: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    fn cursor_position(&self) -> usize {
        let x = self.col_pos;
        let y = self.row_pos;
        x + y * BUFFER_WIDTH
    }

    // TODO: check this
    pub fn enable_cursor(&mut self) {
        let pos = self.cursor_position();
        let mut port_3d4 = Port::new(0x3D4);
        let mut port_3d5 = Port::new(0x3D5);
        unsafe {
            port_3d4.write(0x0A as u8);
            let val = port_3d5.read();
            port_3d5.write(((val & 0xC0) | pos as u8) as u8);
            port_3d4.write(0x0B as u8);
            let val = port_3d5.read();
            port_3d5.write(((val & 0xE0) | pos as u8) as u8);
        }
    }

    pub fn write_cursor(&mut self) {
        let pos = self.cursor_position();
        let mut port_3d4 = Port::new(0x3D4);
        let mut port_3d5 = Port::new(0x3D5);
        unsafe {
            port_3d4.write(0x0F as u8);
            port_3d5.write((pos & 0xFF) as u8);
            port_3d4.write(0x0E as u8);
            port_3d5.write(((pos >> 8) & 0xFF) as u8);
        }
    }

    /// Writes an ASCII byte to the buffer.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            0x0A => { // Newline
                self.new_line();
            },
            0x08 => { // Backspace
                if self.col_pos > 0 {
                    self.col_pos -= 1;
                    let blank = ScreenChar {
                        ascii_character: b' ',
                        color_code: self.color_code,
                    };
                    let x = self.col_pos;
                    let y = self.row_pos;
                    self.buffer.chars[y][x].write(blank);
                }
            },
            byte => {
                if self.col_pos >= BUFFER_WIDTH {
                    self.new_line();
                }

                let x = self.col_pos;
                let y = self.row_pos;
                let color_code = self.color_code;
                self.buffer.chars[y][x].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.col_pos += 1;
            }
        }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7E | 0x08 | 0x0A => {
                    self.write_byte(byte) // Printable chars, backspace, newline
                },
                _ => {
                    self.write_byte(0xFE) // Square
                }
            }
        }
    }

    fn new_line(&mut self) {
        if self.row_pos < BUFFER_HEIGHT - 1 {
            self.row_pos += 1;
        } else {
            for y in 1..BUFFER_HEIGHT {
                for x in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[y][x].read();
                    self.buffer.chars[y - 1][x].write(character);
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
        }
        self.col_pos = 0;
    }

    /// Clears a row by overwriting it with blank characters.
    fn clear_row(&mut self, y: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for x in 0..BUFFER_WIDTH {
            self.buffer.chars[y][x].write(blank);
        }
    }

    pub fn clear_screen(&mut self) {
        for y in 0..BUFFER_HEIGHT {
            self.clear_row(y);
        }
        self.row_pos = 0;
        self.col_pos = 0;
        self.write_cursor();
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        self.write_cursor();
        Ok(())
    }
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::kernel::vga::_print(format_args!($($arg)*)));
}

/// Prints the given formatted string to the VGA text buffer
/// through the global `WRITER` instance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

pub fn clear_screen() {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().clear_screen();
    });
}
