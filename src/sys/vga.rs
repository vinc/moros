use crate::api::font::Font;
use crate::api::vga::{Color, Palette};
use crate::api::vga::color;
use crate::sys;

use bit_field::BitField;
use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use vte::{Params, Parser, Perform};
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

// See https://web.stanford.edu/class/cs140/projects/pintos/specs/freevga/vga/vga.htm
// And https://01.org/sites/default/files/documentation/snb_ihd_os_vol3_part1_0.pdf

const ATTR_ADDR_DATA_REG:      u16 = 0x3C0;
const ATTR_DATA_READ_REG:      u16 = 0x3C1;
const SEQUENCER_ADDR_REG:      u16 = 0x3C4;
const DAC_ADDR_WRITE_MODE_REG: u16 = 0x3C8;
const DAC_DATA_REG:            u16 = 0x3C9;
const GRAPHICS_ADDR_REG:       u16 = 0x3CE;
const CRTC_ADDR_REG:           u16 = 0x3D4;
const CRTC_DATA_REG:           u16 = 0x3D5;
const INPUT_STATUS_REG:        u16 = 0x3DA;

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
    fn writer_position(&self) -> (usize, usize) {
        (self.writer[0], self.writer[1])
    }

    fn set_writer_position(&mut self, x: usize, y: usize) {
        self.writer = [x, y];
    }

    fn cursor_position(&self) -> (usize, usize) {
        (self.cursor[0], self.cursor[1])
    }

    fn set_cursor_position(&mut self, x: usize, y: usize) {
        self.cursor = [x, y];
        self.write_cursor();
    }

    fn write_cursor(&mut self) {
        let pos = self.cursor[0] + self.cursor[1] * BUFFER_WIDTH;
        let mut addr = Port::new(CRTC_ADDR_REG);
        let mut data = Port::new(CRTC_DATA_REG);
        unsafe {
            addr.write(0x0F as u8);
            data.write((pos & 0xFF) as u8);
            addr.write(0x0E as u8);
            data.write(((pos >> 8) & 0xFF) as u8);
        }
    }

    fn disable_cursor(&self) {
        // http://www.osdever.net/FreeVGA/vga/crtcreg.htm#0A
        let mut addr = Port::new(CRTC_ADDR_REG);
        let mut data = Port::new(CRTC_DATA_REG);
        unsafe {
            addr.write(0x0A as u8);
            data.write(0x20 as u8);
        }
    }

    fn enable_cursor(&self) {
        let mut addr: Port<u8> = Port::new(CRTC_ADDR_REG);
        let mut data: Port<u8> = Port::new(CRTC_DATA_REG);
        let cursor_start = 13; // Starting row
        let cursor_end = 14; // Ending row
        unsafe {
            addr.write(0x0A); // Cursor Start Register
            let b = data.read();
            data.write((b & 0xC0) | cursor_start);

            addr.write(0x0B); // Cursor End Register
            let b = data.read();
            data.write((b & 0xE0) | cursor_end);
        }
    }

    fn disable_echo(&self) {
        sys::console::disable_echo();
    }

    fn enable_echo(&self) {
        sys::console::enable_echo();
    }

    fn write_byte(&mut self, byte: u8) {
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
                self.buffer.chars[y - 1] = self.buffer.chars[y];
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
        self.buffer.chars[y][x..BUFFER_WIDTH].fill(c);
    }

    fn clear_screen(&mut self) {
        for y in 0..BUFFER_HEIGHT {
            self.clear_row_after(0, y);
        }
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
        let mut sequencer: Port<u16> = Port::new(SEQUENCER_ADDR_REG);
        let mut graphics: Port<u16> = Port::new(GRAPHICS_ADDR_REG);
        let buffer = 0xA0000 as *mut u8;

        unsafe {
            sequencer.write(0x0100); // do a sync reset
            sequencer.write(0x0402); // write plane 2 only
            sequencer.write(0x0704); // sequetial access
            sequencer.write(0x0300); // end the reset
            graphics.write(0x0204); // read plane 2 only
            graphics.write(0x0005); // disable odd/even
            graphics.write(0x0006); // VRAM at 0xA0000

            for i in 0..font.size as usize {
                for j in 0..font.height as usize {
                    let vga_offset = j + i * 32 as usize;
                    let fnt_offset = j + i * font.height as usize;
                    buffer.add(vga_offset).write_volatile(font.data[fnt_offset]);
                }
            }

            sequencer.write(0x0100); // do a sync reset
            sequencer.write(0x0302); // write plane 0 & 1
            sequencer.write(0x0304); // even/odd access
            sequencer.write(0x0300); // end the reset
            graphics.write(0x0004); // restore to default
            graphics.write(0x1005); // resume odd/even
            graphics.write(0x0E06); // VRAM at 0xB800
        }
    }

    pub fn set_palette(&mut self, palette: Palette) {
        let mut addr: Port<u8> = Port::new(DAC_ADDR_WRITE_MODE_REG);
        let mut data: Port<u8> = Port::new(DAC_DATA_REG);
        for (i, (r, g, b)) in palette.colors.iter().enumerate() {
            if i < 16 {
                let reg = color::from_index(i).to_vga_reg();
                unsafe {
                    addr.write(reg);
                    data.write(vga_color(*r));
                    data.write(vga_color(*g));
                    data.write(vga_color(*b));
                }
            }
        }
    }
}

// Convert 8-bit to 6-bit color
fn vga_color(color: u8) -> u8 {
    color >> 2
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
                // TODO: Don't go past edge
                self.writer[1] -= n;
                self.cursor[1] -= n;
            },
            'B' => { // Cursor Down
                let mut n = 1;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                // TODO: Don't go past edge
                self.writer[1] += n;
                self.cursor[1] += n;
            },
            'C' => { // Cursor Forward
                let mut n = 1;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                // TODO: Don't go past edge
                self.writer[0] += n;
                self.cursor[0] += n;
            },
            'D' => { // Cursor Backward
                let mut n = 1;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                // TODO: Don't go past edge
                self.writer[0] -= n;
                self.cursor[0] -= n;
            },
            'G' => { // Cursor Horizontal Absolute
                let (_, y) = self.cursor_position();
                let mut x = 1;
                for param in params.iter() {
                    x = param[0] as usize; // 1-indexed value
                }
                if x > BUFFER_WIDTH {
                    return;
                }
                self.set_writer_position(x - 1, y);
                self.set_cursor_position(x - 1, y);
            },
            'H' => { // Move cursor
                let mut x = 1;
                let mut y = 1;
                for (i, param) in params.iter().enumerate() {
                    match i {
                        0 => y = param[0] as usize, // 1-indexed value
                        1 => x = param[0] as usize, // 1-indexed value
                        _ => break,
                    };
                }
                if x > BUFFER_WIDTH || y > BUFFER_HEIGHT {
                    return;
                }
                self.set_writer_position(x - 1, y - 1);
                self.set_cursor_position(x - 1, y - 1);
            },
            'J' => { // Erase in Display
                let mut n = 0;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                match n {
                    // TODO: 0 and 1, from cursor to begining or to end of screen
                    2 => self.clear_screen(),
                    _ => return,
                }
                self.set_writer_position(0, 0);
                self.set_cursor_position(0, 0);
            },
            'K' => { // Erase in Line
                let (x, y) = self.cursor_position();
                let mut n = 0;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                match n {
                    0 => self.clear_row_after(x, y),
                    1 => return, // TODO: self.clear_row_before(x, y),
                    2 => self.clear_row_after(0, y),
                    _ => return,
                }
                self.set_writer_position(x, y);
                self.set_cursor_position(x, y);
            },
            'h' => { // Enable
                for param in params.iter() {
                    match param[0] {
                        12 => self.enable_echo(),
                        25 => self.enable_cursor(),
                        _ => return,
                    }
                }
            },
            'l' => { // Disable
                for param in params.iter() {
                    match param[0] {
                        12 => self.disable_echo(),
                        25 => self.disable_cursor(),
                        _ => return,
                    }
                }
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

pub fn cols() -> usize {
    BUFFER_WIDTH
}

pub fn rows() -> usize {
    BUFFER_HEIGHT
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

// 0x00 -> top
// 0x0F -> bottom
// 0x1F -> max (invisible)
fn set_underline_location(location: u8) {
    interrupts::without_interrupts(|| {
        let mut addr: Port<u8> = Port::new(CRTC_ADDR_REG);
        let mut data: Port<u8> = Port::new(CRTC_DATA_REG);
        unsafe {
            addr.write(0x14); // Underline Location Register
            data.write(location);
        }
    })
}

fn set_attr_ctrl_reg(index: u8, value: u8) {
    interrupts::without_interrupts(|| {
        let mut isr: Port<u8> = Port::new(INPUT_STATUS_REG);
        let mut addr: Port<u8> = Port::new(ATTR_ADDR_DATA_REG);
        unsafe {
            isr.read(); // Reset to address mode
            let tmp = addr.read();
            addr.write(index);
            addr.write(value);
            addr.write(tmp);
        }
    })
}

fn get_attr_ctrl_reg(index: u8) -> u8 {
    interrupts::without_interrupts(|| {
        let mut isr: Port<u8> = Port::new(INPUT_STATUS_REG);
        let mut addr: Port<u8> = Port::new(ATTR_ADDR_DATA_REG);
        let mut data: Port<u8> = Port::new(ATTR_DATA_READ_REG);
        let index = index | 0x20; // Set "Palette Address Source" bit
        unsafe {
            isr.read(); // Reset to address mode
            let tmp = addr.read();
            addr.write(index);
            let res = data.read();
            addr.write(tmp);
            res
        }
    })
}

pub fn init() {
    // Map palette registers to color registers
    set_attr_ctrl_reg(0x0, 0x00);
    set_attr_ctrl_reg(0x1, 0x01);
    set_attr_ctrl_reg(0x2, 0x02);
    set_attr_ctrl_reg(0x3, 0x03);
    set_attr_ctrl_reg(0x4, 0x04);
    set_attr_ctrl_reg(0x5, 0x05);
    set_attr_ctrl_reg(0x6, 0x14);
    set_attr_ctrl_reg(0x7, 0x07);
    set_attr_ctrl_reg(0x8, 0x38);
    set_attr_ctrl_reg(0x9, 0x39);
    set_attr_ctrl_reg(0xA, 0x3A);
    set_attr_ctrl_reg(0xB, 0x3B);
    set_attr_ctrl_reg(0xC, 0x3C);
    set_attr_ctrl_reg(0xD, 0x3D);
    set_attr_ctrl_reg(0xE, 0x3E);
    set_attr_ctrl_reg(0xF, 0x3F);

    set_palette(Palette::default());

    // Disable blinking
    let reg = 0x10; // Attribute Mode Control Register
    let mut attr = get_attr_ctrl_reg(reg);
    attr.set_bit(3, false); // Clear "Blinking Enable" bit
    set_attr_ctrl_reg(reg, attr);

    set_underline_location(0x1F); // Disable underline

    WRITER.lock().clear_screen();
}
