use crate::api::font::Font;
use crate::api::vga::{Color, Palette};
use crate::api::vga::color;
use bit_field::BitField;
use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use vte::{Params, Parser, Perform};
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

// See: https://web.stanford.edu/class/cs140/projects/pintos/specs/freevga/vga/vga.htm

const FG: Color = Color::LightGray;
const BG: Color = Color::Black;
const UNPRINTABLE: u8 = 0x00; // Unprintable chars will be replaced by this one

lazy_static! {
    pub static ref PARSER: Mutex<Parser> = Mutex::new(Parser::new());
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        cursor: [0; 2],
        writer: [0; 2],
        color_code: ColorCode::new(FG, BG),
        buffer: unsafe { &mut *(0xB8000 as *mut Buffer) },
    });
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
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
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
        let mut crtc_ontroller_address_register = Port::new(0x3D4);
        let mut crtc_ontroller_data_register = Port::new(0x3D5);
        unsafe {
            crtc_ontroller_address_register.write(0x0A as u8);
            let val = crtc_ontroller_data_register.read();
            crtc_ontroller_data_register.write(((val & 0xC0) | pos as u8) as u8);
            crtc_ontroller_address_register.write(0x0B as u8);
            let val = crtc_ontroller_data_register.read();
            crtc_ontroller_data_register.write(((val & 0xE0) | pos as u8) as u8);
        }
    }

    pub fn write_cursor(&mut self) {
        let pos = self.cursor[0] + self.cursor[1] * BUFFER_WIDTH;
        let mut crtc_ontroller_address_register = Port::new(0x3D4);
        let mut crtc_ontroller_data_register = Port::new(0x3D5);
        unsafe {
            crtc_ontroller_address_register.write(0x0F as u8);
            crtc_ontroller_data_register.write((pos & 0xFF) as u8);
            crtc_ontroller_address_register.write(0x0E as u8);
            crtc_ontroller_data_register.write(((pos >> 8) & 0xFF) as u8);
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
                    let c = ScreenChar {
                        ascii_code: b' ',
                        color_code: self.color_code,
                    };
                    let x = self.writer[0];
                    let y = self.writer[1];
                    unsafe {
                        core::ptr::write_volatile(&mut self.buffer.chars[y][x], c);
                    }
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
                let c = ScreenChar { ascii_code, color_code };
                unsafe {
                    core::ptr::write_volatile(&mut self.buffer.chars[y][x], c);
                }
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
                    unsafe {
                        let c = core::ptr::read_volatile(&mut self.buffer.chars[y][x]);
                        core::ptr::write_volatile(&mut self.buffer.chars[y - 1][x], c);
                    }
                }
            }
            self.clear_row_after(0, BUFFER_HEIGHT - 1);
        }
        self.writer[0] = 0;
    }

    fn clear_row_after(&mut self, x: usize, y: usize) {
        let c = ScreenChar {
            ascii_code: b' ',
            color_code: self.color_code,
        };
        for i in x..BUFFER_WIDTH {
            unsafe {
                core::ptr::write_volatile(&mut self.buffer.chars[y][i], c);
            }
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
        let fg = color::from_index(cc.get_bits(0..4) as usize);
        let bg = color::from_index(cc.get_bits(4..8) as usize);
        (fg, bg)
    }

    // See: https://slideplayer.com/slide/3888880
    pub fn set_font(&mut self, font: &Font) {
        let mut sequencer_address_register: Port<u16> = Port::new(0x3C4);
        let mut graphics_controller_address_register: Port<u16> = Port::new(0x3CE);
        let buffer = 0xA0000 as *mut u8;

        unsafe {
            sequencer_address_register.write(0x0100); // do a sync reset
            sequencer_address_register.write(0x0402); // write plane 2 only
            sequencer_address_register.write(0x0704); // sequetial access
            sequencer_address_register.write(0x0300); // end the reset
            graphics_controller_address_register.write(0x0204); // read plane 2 only
            graphics_controller_address_register.write(0x0005); // disable odd/even
            graphics_controller_address_register.write(0x0006); // VRAM at 0xA0000

            for i in 0..font.size as usize {
                for j in 0..font.height as usize {
                    let vga_offset = j + i * 32 as usize;
                    let fnt_offset = j + i * font.height as usize;
                    buffer.add(vga_offset).write_volatile(font.data[fnt_offset]);
                }
            }

            sequencer_address_register.write(0x0100); // do a sync reset
            sequencer_address_register.write(0x0302); // write plane 0 & 1
            sequencer_address_register.write(0x0304); // even/odd access
            sequencer_address_register.write(0x0300); // end the reset
            graphics_controller_address_register.write(0x0004); // restore to default
            graphics_controller_address_register.write(0x1005); // resume odd/even
            graphics_controller_address_register.write(0x0E06); // VRAM at 0xB800
        }
    }

    pub fn set_palette(&mut self, palette: Palette) {
        let mut addr: Port<u8> = Port::new(0x03C8); // Address Write Mode Register
        let mut data: Port<u8> = Port::new(0x03C9); // Data Register
        for (i, r, g, b) in palette.colors {
            if i < 16 {
                let code = color::from_index(i as usize).to_palette_code();
                unsafe {
                    addr.write(code);
                    data.write(r >> 2); // Convert 8-bit color to 6-bit color
                    data.write(g >> 2);
                    data.write(b >> 2);
                }
            }
        }
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
        match c {
            'm' => {
                let mut fg = FG;
                let mut bg = BG;
                for param in params.iter() {
                    match param[0] {
                        0 => {
                            fg = FG;
                            bg = BG;
                        },
                        30..=37 | 90..=97 => {
                            fg = color::from_ansi(param[0] as u8);
                        },
                        40..=47 | 100..=107 => {
                            bg = color::from_ansi((param[0] as u8) - 10);
                        },
                        _ => {},
                    }
                }
                self.set_color(fg, bg);
            },
            'A' => { // Cursor Up
                let mut n = 1;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                self.writer[1] += n;
                self.cursor[1] += n;
            },
            'B' => { // Cursor Down
                let mut n = 1;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                self.writer[1] -= n;
                self.cursor[1] -= n;
            },
            'C' => { // Cursor Forward
                let mut n = 1;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                self.writer[0] += n;
                self.cursor[0] += n;
            },
            'D' => { // Cursor Backward
                let mut n = 1;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                self.writer[0] -= n;
                self.cursor[0] -= n;
            },
            _ => {},
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut parser = PARSER.lock();
        for byte in s.bytes() {
            parser.advance(self, byte);
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

pub fn cols() -> usize {
    BUFFER_WIDTH
}

pub fn rows() -> usize {
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

// ASCII Printable
// Backspace
// New Line
// Carriage Return
// Extended ASCII Printable
pub fn is_printable(c: u8) -> bool {
    matches!(c, 0x20..=0x7E | 0x08 | 0x0A | 0x0D | 0x7F..=0xFF)
}

pub fn set_font(font: &Font) {
    interrupts::without_interrupts(|| {
        WRITER.lock().set_font(font);
    })
}

pub fn set_palette(palette: Palette) {
    interrupts::without_interrupts(|| {
        WRITER.lock().set_palette(palette)
    })
}

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
}
