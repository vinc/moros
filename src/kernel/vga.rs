use crate::kernel;
use bit_field::BitField;
use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use vte::{Params, Parser, Perform};
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;
use vga::vga::VGA;

const FG: Color = Color::LightGray;
const BG: Color = Color::Black;
const UNPRINTABLE: u8 = 0xFE; // Unprintable characters will be replaced by a square

lazy_static! {
    pub static ref PARSER: Mutex<Parser> = Mutex::new(Parser::new());
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        cursor: [0; 2],
        writer: [0; 2],
        color_code: ColorCode::new(FG, BG),
        buffer: unsafe { &mut *(0xB8000 as *mut Buffer) },
    });
}

/// The standard color palette in VGA text mode
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

fn color_from_ansi(code: u8) -> Color {
    match code {
        30 => Color::Black,
        31 => Color::Red,
        32 => Color::Green,
        33 => Color::Brown,
        34 => Color::Blue,
        35 => Color::Magenta,
        36 => Color::Cyan,
        37 => Color::LightGray,
        90 => Color::DarkGray,
        91 => Color::LightRed,
        92 => Color::LightGreen,
        93 => Color::Yellow,
        94 => Color::LightBlue,
        95 => Color::Pink,
        96 => Color::LightCyan,
        97 => Color::White,
        _ => FG, // Error
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_code: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

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

    /// Writes an ASCII byte to the screen buffer
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
                        ascii_code: b' ',
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
                let ascii_code = if is_printable(byte) { byte } else { UNPRINTABLE };
                let color_code = self.color_code;
                self.buffer.chars[y][x].write(ScreenChar { ascii_code, color_code });
                self.writer[0] += 1;
            }
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
            self.clear_row_after(0, BUFFER_HEIGHT - 1);
        }
        self.writer[0] = 0;
    }

    fn clear_row_after(&mut self, x: usize, y: usize) {
        let blank = ScreenChar {
            ascii_code: b' ',
            color_code: self.color_code,
        };
        for i in x..BUFFER_WIDTH {
            self.buffer.chars[y][i].write(blank);
        }
    }

    pub fn clear_screen(&mut self) {
        for y in 0..BUFFER_HEIGHT {
            self.clear_row_after(0, y);
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

/// See https://vt100.net/emu/dec_ansi_parser
impl Perform for Writer {
    fn print(&mut self, c: char) {
        self.write_byte(c as u8);
    }

    fn execute(&mut self, byte: u8) {
        self.write_byte(byte);
    }

    fn csi_dispatch(&mut self, params: &Params, _: &[u8], _: bool, c: char) {
        if c == 'm' {
            let mut fg = FG;
            let mut bg = BG;
            for param in params.iter() {
                match param[0] {
                    0 => {
                        fg = FG;
                        bg = BG;
                    },
                    30..=37 | 90..=97 => {
                        fg = color_from_ansi(param[0] as u8);
                    },
                    40..=47 | 100..=107 => {
                        bg = color_from_ansi((param[0] as u8) - 10);
                    },
                    _ => {}
                }
            }
            self.set_color(fg, bg);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut state_machine = PARSER.lock();
        for byte in s.bytes() {
            state_machine.advance(self, byte);
        }
        let (x, y) = self.writer_position();
        self.set_cursor_position(x, y);
        Ok(())
    }
}

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
    clear_row_after(0);
}

pub fn clear_row_after(x: usize) {
    let (_, y) = writer_position();
    interrupts::without_interrupts(|| {
        WRITER.lock().clear_row_after(x, y);
    });
    set_writer_position(x, y);
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

pub fn set_font(name: &str) {
    if let Some(font) = kernel::fonts::vga_font(name) {
        VGA.lock().load_font(font)
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

    set_font("zap-light-8x16");

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
