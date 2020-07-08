use crate::kernel;
use bit_field::BitField;
use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use vte;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

const FG: Color = Color::LightGray;
const BG: Color = Color::Black;

lazy_static! {
    /// A global `Writer` instance that can be used for printing to the VGA text buffer.
    ///
    /// Used by the `print!` and `println!` macros.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        cursor: [0; 2],
        writer: [0; 2],
        color_code: ColorCode::new(FG, BG),
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

const COLORS: [Color; 16] = [
    Color::Black,
    Color::Blue,
    Color::Green,
    Color::Cyan,
    Color::Red,
    Color::Magenta,
    Color::Brown,
    Color::LightGray,
    Color::DarkGray,
    Color::LightBlue,
    Color::LightGreen,
    Color::LightCyan,
    Color::LightRed,
    Color::Pink,
    Color::Yellow,
    Color::White,
];

pub fn color_from_ansi(code: u8) -> Color {
    match code {
        30 => Color::Black,
        31 => Color::Red,
        32 => Color::Green,
        33 => Color::Brown,
        34 => Color::Blue,
        35 => Color::Magenta,
        36 => Color::Cyan,
        37 => Color::LightGray,
        38 => Color::DarkGray,
        39 => Color::LightRed,
        40 => Color::LightGreen,
        41 => Color::Yellow,
        42 => Color::LightBlue,
        43 => Color::Pink,
        44 => Color::LightCyan,
        45 => Color::White,
        _ => FG, // Error
    }
}

pub fn color_to_ansi(color: Color) -> u8 {
    match color {
        Color::Black => 30,
        Color::Red => 31,
        Color::Green => 32,
        Color::Brown => 33,
        Color::Blue => 34,
        Color::Magenta => 35,
        Color::Cyan => 36,
        Color::LightGray => 37,
        Color::DarkGray => 38,
        Color::LightRed => 39,
        Color::LightGreen => 40,
        Color::Yellow => 41,
        Color::LightBlue => 42,
        Color::Pink => 43,
        Color::LightCyan => 44,
        Color::White => 45,
    }
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
    cursor: [usize; 2], // x, y
    writer: [usize; 2], // x, y
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn writer_position(&self) -> (usize, usize) {
        (self.writer[0], self.writer[1])
    }

    pub fn set_writer_position(&mut self, x: usize, y: usize) {
        self.writer = [x, y];
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor[0], self.cursor[1])
    }

    pub fn set_cursor_position(&mut self, x: usize, y: usize) {
        self.cursor = [x, y];
        self.write_cursor();
    }

    // TODO: check this
    pub fn enable_cursor(&mut self) {
        let pos = self.cursor[0] + self.cursor[1] * BUFFER_WIDTH;
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
        let pos = self.cursor[0] + self.cursor[1] * BUFFER_WIDTH;
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
            0x0D => { // Carriage Return
            },
            0x08 => { // Backspace
                if self.writer[0] > 0 {
                    self.writer[0] -= 1;
                    let blank = ScreenChar {
                        ascii_character: b' ',
                        color_code: self.color_code,
                    };
                    let x = self.writer[0];
                    let y = self.writer[1];
                    self.buffer.chars[y][x].write(blank);
                }
            },
            byte => {
                if self.writer[0] >= BUFFER_WIDTH {
                    self.new_line();
                }

                let x = self.writer[0];
                let y = self.writer[1];
                let color_code = self.color_code;
                self.buffer.chars[y][x].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.writer[0] += 1;
            }
        }
    }

    fn write_string(&mut self, s: &str) {
        let mut state_machine = vte::Parser::new();
        for byte in s.bytes() {
            state_machine.advance(self, byte);
        }
    }

    fn new_line(&mut self) {
        if self.writer[1] < BUFFER_HEIGHT - 1 {
            self.writer[1] += 1;
        } else {
            for y in 1..BUFFER_HEIGHT {
                for x in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[y][x].read();
                    self.buffer.chars[y - 1][x].write(character);
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
        }
        self.writer[0] = 0;

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
        self.set_writer_position(0, 0);
        self.set_cursor_position(0, 0);
    }

    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    pub fn color(&self) -> (Color, Color) {
        let cc = self.color_code.0;
        let fg = COLORS[cc.get_bits(0..4) as usize];
        let bg = COLORS[cc.get_bits(4..8) as usize];
        (fg, bg)
    }
}

impl vte::Perform for Writer {
    fn print(&mut self, c: char) {
        let byte = c as u8;
        if is_printable(byte) {
            self.write_byte(byte) // Printable chars, backspace, newline
        } else {
            self.write_byte(0xFE) // Square
        }
    }

    fn execute(&mut self, byte: u8) {
        self.write_byte(byte);
        kernel::serial::print_fmt(format_args!("[execute] {:02x}\n", byte));
    }

    fn hook(&mut self, params: &[i64], intermediates: &[u8], ignore: bool, c: char) {
        kernel::serial::print_fmt(format_args!("[hook] params={:?}, intermediates={:?}, ignore={:?}, char={:?}\n", params, intermediates, ignore, c));
    }

    fn put(&mut self, byte: u8) {
        kernel::serial::print_fmt(format_args!("[put] {:02x}\n", byte));
    }

    fn unhook(&mut self) {
        kernel::serial::print_fmt(format_args!("[unhook]\n"));
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        kernel::serial::print_fmt(format_args!("[osc_dispatch] params={:?} bell_terminated={}\n", params, bell_terminated));
    }

    fn csi_dispatch(&mut self, params: &[i64], intermediates: &[u8], ignore: bool, c: char) {
        if c == 'm' {
            let mut fg = FG;
            let mut bg = BG;
            if params.len() > 0 && params[0] >= 30 {
                fg = color_from_ansi(params[0] as u8);
                if params.len() > 1 {
                    bg = color_from_ansi(params[1] as u8);
                }
            }
            self.set_color(fg, bg);
        }

        kernel::serial::print_fmt(format_args!("[csi_dispatch] params={:?}, intermediates={:?}, ignore={:?}, char={:?}\n", params, intermediates, ignore, c));
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        kernel::serial::print_fmt(format_args!("[esc_dispatch] intermediates={:?}, ignore={:?}, byte={:02x}\n", intermediates, ignore, byte));
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        let (x, y) = self.writer_position();
        self.set_cursor_position(x, y);
        Ok(())
    }
}

/// Prints the given formatted string to the VGA text buffer
/// through the global `WRITER` instance.
#[doc(hidden)]
pub fn print_fmt(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).expect("Could not print to VGA");
    });
}

pub fn clear_screen() {
    interrupts::without_interrupts(|| {
        WRITER.lock().clear_screen();
    });
}

pub fn clear_row() {
    let (_, y) = writer_position();
    interrupts::without_interrupts(|| {
        WRITER.lock().clear_row(y);
    });
    set_writer_position(0, y);
}

pub fn screen_width() -> usize {
    BUFFER_WIDTH
}

pub fn screen_height() -> usize {
    BUFFER_HEIGHT
}

pub fn set_cursor_position(x: usize, y: usize) {
    interrupts::without_interrupts(|| {
        WRITER.lock().set_cursor_position(x, y);
    });
}

pub fn set_writer_position(x: usize, y: usize) {
    interrupts::without_interrupts(|| {
        WRITER.lock().set_writer_position(x, y);
    });
}

pub fn cursor_position() -> (usize, usize) {
    interrupts::without_interrupts(|| {
        WRITER.lock().cursor_position()
    })
}

pub fn writer_position() -> (usize, usize) {
    interrupts::without_interrupts(|| {
        WRITER.lock().writer_position()
    })
}

pub fn color() -> (Color, Color) {
    interrupts::without_interrupts(|| {
        WRITER.lock().color()
    })
}

pub fn set_color(foreground: Color, background: Color) {
    interrupts::without_interrupts(|| {
        WRITER.lock().set_color(foreground, background)
    })
}

pub fn colors() -> [Color; 16] {
    COLORS
}

// Printable ascii chars + backspace + newline
pub fn is_printable(c: u8) -> bool {
    match c {
        0x20..=0x7E | 0x08 | 0x0A | 0x0D => true,
        _ => false,
    }
}

// Dark Gruvbox color palette
const PALETTE: [(u8, u8, u8, u8); 16] = [
    (0x00, 0x28, 0x28, 0x28), // Black
    (0x01, 0x45, 0x85, 0x88), // Blue
    (0x02, 0x98, 0x97, 0x1A), // Green
    (0x03, 0x68, 0x9D, 0x6A), // Cyan
    (0x04, 0xCC, 0x24, 0x1D), // Red
    (0x05, 0xB1, 0x62, 0x86), // Magenta
    (0x07, 0xEB, 0xDB, 0xB2), // Light Gray
    (0x14, 0xD7, 0x99, 0x21), // Brown (Dark Yellow)
    (0x38, 0xA8, 0x99, 0x84), // Gray (Dark Gray)
    (0x39, 0x83, 0xa5, 0x98), // Light Blue
    (0x3A, 0xB8, 0xBB, 0x26), // Light Green
    (0x3B, 0x8E, 0xC0, 0x7C), // Light Cyan
    (0x3C, 0xFB, 0x49, 0x34), // Light Red
    (0x3D, 0xD3, 0x86, 0x9B), // Pink (Light Magenta)
    (0x3E, 0xFA, 0xBD, 0x2F), // Yellow (Light Yellow)
    (0x3F, 0xFB, 0xF1, 0xF7), // White
];

pub fn init() {
    let mut isr: Port<u8> = Port::new(0x03DA); // Input Status Register
    let mut aadr: Port<u8> = Port::new(0x03C0); // Attribute Address/Data Register
    let mut adrr: Port<u8> = Port::new(0x03C1); // Attribute Data Read Register

    // Disable blinking
    unsafe {
        isr.read(); // Reset to address mode
        aadr.write(0x30); // Select attribute mode control register
        let value = adrr.read(); // Read attribute mode control register
        aadr.write(value & !0x08); // Use `value | 0x08` to enable and `value ^ 0x08` to toggle
    }

    // Load color palette
    let mut addr: Port<u8> = Port::new(0x03C8); // Address Write Mode Register
    let mut data: Port<u8> = Port::new(0x03C9); // Data Register
    for (i, r, g, b) in &PALETTE {
        unsafe {
            addr.write(*i);
            data.write(*r >> 2); // Convert 8-bit color to 6-bit color
            data.write(*g >> 2);
            data.write(*b >> 2);
        }
    }
}
