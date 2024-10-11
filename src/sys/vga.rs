use crate::api::font::Font;
use crate::api::fs::{FileIO, IO};
use crate::api::vga::color;
use crate::api::vga::{Color, Palette};
use crate::sys;

use alloc::string::String;
use bit_field::BitField;
use core::convert::TryFrom;
use core::cmp;
use core::fmt;
use core::fmt::Write;
use core::num::ParseIntError;
use lazy_static::lazy_static;
use spin::Mutex;
use vte::{Params, Parser, Perform};
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

const ATTR_ADDR_REG:           u16 = 0x3C0;
const ATTR_WRITE_REG:          u16 = 0x3C0;
const ATTR_READ_REG:           u16 = 0x3C1;
const MISC_WRITE_REG:          u16 = 0x3C2;
const SEQUENCER_ADDR_REG:      u16 = 0x3C4;
const SEQUENCER_DATA_REG:      u16 = 0x3C5;
const DAC_ADDR_WRITE_MODE_REG: u16 = 0x3C8;
const DAC_DATA_REG:            u16 = 0x3C9;
const GRAPHICS_ADDR_REG:       u16 = 0x3CE;
const GRAPHICS_DATA_REG:       u16 = 0x3CF;
const CRTC_ADDR_REG:           u16 = 0x3D4;
const CRTC_DATA_REG:           u16 = 0x3D5;
const INPUT_STATUS_REG:        u16 = 0x3DA;
const INSTAT_READ_REG:         u16 = 0x3DA;

// Source: https://www.singlix.com/trdos/archive/vga/Graphics%20in%20pmode.pdf
const T_80_25: [u8; 61] = [
    // MISC
    0x67,
    // SEQ
    0x03, 0x00, 0x03, 0x00, 0x02,
    // CRTC
    0x5F, 0x4F, 0x50, 0x82, 0x55, 0x81, 0xBF, 0x1F, 0x00, 0x4F, 0x0D, 0x0E,
    0x00, 0x00, 0x00, 0x50, 0x9C, 0x0E, 0x8F, 0x28, 0x1F, 0x96, 0xB9, 0xA3,
    0xFF,
    // GC
    0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x0E, 0x00, 0xFF,
    // AC
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x14, 0x07, 0x38, 0x39, 0x3A, 0x3B,
    0x3C, 0x3D, 0x3E, 0x3F, 0x0C, 0x00, 0x0F, 0x08, 0x00
];

const G_320_200_256: [u8; 61] = [
    // MISC
    0x63,
    // SEQ
    0x03, 0x01, 0x0F, 0x00, 0x0E,
    // CRTC
    0x5F, 0x4F, 0x50, 0x82, 0x54, 0x80, 0xBF, 0x1F, 0x00, 0x41, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x9C, 0x0E, 0x8F, 0x28, 0x40, 0x96, 0xB9, 0xA3,
    0xFF,
    // GC
    0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x05, 0x0F, 0xFF,
    // AC
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
    0x0C, 0x0D, 0x0E, 0x0F, 0x41, 0x00, 0x0F, 0x00, 0x00
];

const G_640_480_16: [u8; 61] = [
    // MISC
    0xE3,
    // SEQ
    0x03, 0x01, 0x08, 0x00, 0x06,
    // CRTC
    0x5F, 0x4F, 0x50, 0x82, 0x54, 0x80, 0x0B, 0x3E, 0x00, 0x40, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0xEA, 0x0C, 0xDF, 0x28, 0x00, 0xE7, 0x04, 0xE3,
    0xFF,
    // GC
    0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x05, 0x0F, 0xFF,
    // AC
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x14, 0x07, 0x38, 0x39, 0x3A, 0x3B,
    0x3C, 0x3D, 0x3E, 0x3F, 0x01, 0x00, 0x0F, 0x00, 0x00
];

const SEQ_REGS_COUNT: usize = 5;
const CRTC_REGS_COUNT: usize = 25;
const GC_REGS_COUNT: usize = 9;
const AC_REGS_COUNT: usize = 21;

pub fn set_80x25_mode() {
    set_mode(&T_80_25);
}

pub fn set_320x200_mode() {
    set_mode(&G_320_200_256);
}

pub fn set_640x480_mode() {
    set_mode(&G_640_480_16);
}

// Source: https://www.singlix.com/trdos/archive/vga/Graphics%20in%20pmode.pdf
fn set_mode(regs: &[u8]) {
    interrupts::without_interrupts(|| {
        let mut misc_write: Port<u8> = Port::new(MISC_WRITE_REG);
        let mut crtc_addr: Port<u8> = Port::new(CRTC_ADDR_REG);
        let mut crtc_data: Port<u8> = Port::new(CRTC_DATA_REG);
        let mut seq_addr: Port<u8> = Port::new(SEQUENCER_ADDR_REG);
        let mut seq_data: Port<u8> = Port::new(SEQUENCER_DATA_REG);
        let mut gc_addr: Port<u8> = Port::new(GRAPHICS_ADDR_REG);
        let mut gc_data: Port<u8> = Port::new(GRAPHICS_DATA_REG);
        let mut ac_addr: Port<u8> = Port::new(ATTR_ADDR_REG);
        let mut ac_write: Port<u8> = Port::new(ATTR_WRITE_REG);
        let mut instat_read: Port<u8> = Port::new(INSTAT_READ_REG);

        let mut regs = regs.to_vec();
        let mut i = 0;

        unsafe {
            misc_write.write(regs[i]);
            i += 1;

            for j in 0..SEQ_REGS_COUNT {
                seq_addr.write(j as u8);
                seq_data.write(regs[i]);
                i += 1;
            }

            // Unlock CRTC regs
            crtc_addr.write(0x03);
            let data = crtc_data.read();
            crtc_data.write(data | 0x80);
            crtc_addr.write(0x11);
            let data = crtc_data.read();
            crtc_data.write(data & !0x80);

            // Keep them unlocked
            regs[0x03] |= 0x80;
            regs[0x11] &= !0x80;

            for j in 0..CRTC_REGS_COUNT {
                crtc_addr.write(j as u8);
                crtc_data.write(regs[i]);
                i += 1;
            }

            for j in 0..GC_REGS_COUNT {
                gc_addr.write(j as u8);
                gc_data.write(regs[i]);
                i += 1;
            }

            for j in 0..AC_REGS_COUNT {
                instat_read.read();
                ac_addr.write(j as u8);
                ac_write.write(regs[i]);
                i += 1;
            }

            // Lock 16-color palette and unblank display
            instat_read.read();
            ac_addr.write(0x20);
        }
    });
}

const FG: Color = Color::DarkWhite;
const BG: Color = Color::DarkBlack;
const UNPRINTABLE: u8 = 0x00; // Unprintable chars will be replaced by this one

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

impl ScreenChar {
    fn new() -> Self {
        Self {
            ascii_code: b' ',
            color_code: ColorCode::new(FG, BG),
        }
    }
}

const SCREEN_WIDTH: usize = 80;
const SCREEN_HEIGHT: usize = 25;
const SCROLL_HEIGHT: usize = 250;

#[repr(transparent)]
struct ScreenBuffer {
    chars: [[ScreenChar; SCREEN_WIDTH]; SCREEN_HEIGHT],
}

lazy_static! {
    pub static ref PARSER: Mutex<Parser> = Mutex::new(Parser::new());
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        cursor: [0; 2],
        writer: [0; 2],
        color_code: ColorCode::new(FG, BG),
        screen_buffer: unsafe { &mut *(0xB8000 as *mut ScreenBuffer) },
        scroll_buffer: [[ScreenChar::new(); SCREEN_WIDTH]; SCROLL_HEIGHT],
        scroll_reader: 0,
        scroll_bottom: SCREEN_HEIGHT,
    });
}

pub struct Writer {
    cursor: [usize; 2], // x, y
    writer: [usize; 2], // x, y
    color_code: ColorCode,
    screen_buffer: &'static mut ScreenBuffer,
    scroll_buffer: [[ScreenChar; SCREEN_WIDTH]; SCROLL_HEIGHT],
    scroll_reader: usize, // Top of the screen
    scroll_bottom: usize, // Bottom of the buffer
}

// Scroll Buffer
// +----------------------------+
// | line 01                    |
// | line 02                    |
// | line 03                    |
// | line 04                    |
// +----------------------------+
// | line 05                    | <-- scroll_reader
// | line 06                    |
// | line 07                    |
// | line 08                    |
// +----------------------------+
// | line 09                    |
// | line 10                    |
// | line 11                    |
// | line 12                    | <-- scroll_bottom
// |                            |
// |                            |
// |                            |
// |                            |
// +----------------------------+
//
// Screen Buffer
// +----------------------------+
// | line 05                    |
// | line 06                    |
// | line 07                    |
// | line 08                    |
// +----------------------------+

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
        let pos = self.cursor[0] + self.cursor[1] * SCREEN_WIDTH;
        let mut addr = Port::new(CRTC_ADDR_REG);
        let mut data = Port::new(CRTC_DATA_REG);
        unsafe {
            addr.write(0x0F as u8);
            data.write((pos & 0xFF) as u8);
            addr.write(0x0E as u8);
            data.write(((pos >> 8) & 0xFF) as u8);
        }
    }

    // Source: http://www.osdever.net/FreeVGA/vga/crtcreg.htm#0A
    fn disable_cursor(&self) {
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
        if self.is_scrolling() {
            // Scroll to the current screen
            self.scroll_reader = self.scroll_bottom - SCREEN_HEIGHT;
            self.scroll();
        }

        match byte {
            0x0A => {
                // Newline
                self.new_line();
            }
            0x0D => { // Carriage Return
            }
            0x08 => {
                // Backspace
                if self.writer[0] > 0 {
                    self.writer[0] -= 1;
                    let c = ScreenChar {
                        ascii_code: b' ',
                        color_code: self.color_code,
                    };
                    let x = self.writer[0];
                    let y = self.writer[1];
                    let ptr = &mut self.screen_buffer.chars[y][x];
                    unsafe { core::ptr::write_volatile(ptr, c); }

                    let dy = self.scroll_reader;
                    self.scroll_buffer[y + dy][x] = c;
                }
            }
            byte => {
                if self.writer[0] >= SCREEN_WIDTH {
                    self.new_line();
                }

                let x = self.writer[0];
                let y = self.writer[1];
                let ascii_code = if is_printable(byte) {
                    byte
                } else {
                    UNPRINTABLE
                };
                let color_code = self.color_code;
                let c = ScreenChar {
                    ascii_code,
                    color_code,
                };
                let ptr = &mut self.screen_buffer.chars[y][x];
                unsafe { core::ptr::write_volatile(ptr, c); }
                self.writer[0] += 1;

                let dy = self.scroll_reader;
                self.scroll_buffer[y + dy][x] = c;
            }
        }
    }

    fn new_line(&mut self) {
        if self.writer[1] < SCREEN_HEIGHT - 1 {
            self.writer[1] += 1;
        } else {
            for y in 1..SCREEN_HEIGHT {
                self.screen_buffer.chars[y - 1] = self.screen_buffer.chars[y];
            }
            if self.scroll_bottom == SCROLL_HEIGHT - 1 {
                for y in 1..SCROLL_HEIGHT {
                    self.scroll_buffer[y - 1] = self.scroll_buffer[y];
                }
            } else {
                self.scroll_reader += 1;
                self.scroll_bottom += 1;
            }
            self.clear_row_after(0, SCREEN_HEIGHT - 1);
        }
        self.writer[0] = 0;
    }

    fn clear_row_after(&mut self, x: usize, y: usize) {
        let c = ScreenChar {
            ascii_code: b' ',
            color_code: self.color_code,
        };
        self.screen_buffer.chars[y][x..SCREEN_WIDTH].fill(c);

        let dy = self.scroll_reader;
        self.scroll_buffer[y + dy][x..SCREEN_WIDTH].fill(c);
    }

    fn clear_screen(&mut self) {
        self.scroll_reader = 0;
        self.scroll_bottom = SCREEN_HEIGHT;
        for y in 0..SCREEN_HEIGHT {
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

    // Source: https://slideplayer.com/slide/3888880
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
                    let ptr = buffer.add(vga_offset);
                    ptr.write_volatile(font.data[fnt_offset]);
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

    pub fn set_palette(&mut self, i: usize, r: u8, g: u8, b: u8) {
        let mut addr: Port<u8> = Port::new(DAC_ADDR_WRITE_MODE_REG);
        let mut data: Port<u8> = Port::new(DAC_DATA_REG);
        //if i < 16 {
            let reg = if i < 16 { color::from_index(i).to_vga_reg() } else { i as u8 };
            unsafe {
                addr.write(reg);
                data.write(vga_color(r));
                data.write(vga_color(g));
                data.write(vga_color(b));
            }
        //}
    }

    fn scroll_up(&mut self, n: usize) {
        self.scroll_reader = self.scroll_reader.saturating_sub(n);
        self.scroll();
    }

    fn scroll_down(&mut self, n: usize) {
        self.scroll_reader = cmp::min(
            self.scroll_reader + n,
            self.scroll_bottom - SCREEN_HEIGHT
        );
        self.scroll();
    }

    fn scroll(&mut self) {
        let dy = self.scroll_reader;
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let c = self.scroll_buffer[y + dy][x];
                let ptr = &mut self.screen_buffer.chars[y][x];
                unsafe { core::ptr::write_volatile(ptr, c); }
            }
        }
        if self.is_scrolling() {
            self.disable_cursor();
        } else {
            self.enable_cursor();
        }
    }

    fn is_scrolling(&self) -> bool {
        // If the current screen is reached we are not scrolling anymore
        self.scroll_reader != self.scroll_bottom - SCREEN_HEIGHT
    }
}

// Convert 8-bit to 6-bit color
fn vga_color(color: u8) -> u8 {
    color >> 2
}

fn parse_palette(palette: &str) -> Result<(usize, u8, u8, u8), ParseIntError> {
    debug_assert!(palette.len() == 8);
    debug_assert!(palette.starts_with('P'));

    let i = usize::from_str_radix(&palette[1..2], 16)?;
    let r = u8::from_str_radix(&palette[2..4], 16)?;
    let g = u8::from_str_radix(&palette[4..6], 16)?;
    let b = u8::from_str_radix(&palette[6..8], 16)?;

    Ok((i, r, g, b))
}

/// Source: https://vt100.net/emu/dec_ansi_parser
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
                        }
                        30..=37 | 90..=97 => {
                            fg = color::from_ansi(param[0] as u8);
                        }
                        40..=47 | 100..=107 => {
                            bg = color::from_ansi((param[0] as u8) - 10);
                        }
                        _ => {}
                    }
                }
                self.set_color(fg, bg);
            }
            'A' => { // Cursor Up
                let mut n = 1;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                self.writer[1] = self.writer[1].saturating_sub(n);
                self.cursor[1] = self.cursor[1].saturating_sub(n);
            }
            'B' => { // Cursor Down
                let mut n = 1;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                let height = SCREEN_HEIGHT - 1;
                self.writer[1] = cmp::min(self.writer[1] + n, height);
                self.cursor[1] = cmp::min(self.cursor[1] + n, height);
            }
            'C' => { // Cursor Forward
                let mut n = 1;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                let width = SCREEN_WIDTH - 1;
                self.writer[0] = cmp::min(self.writer[0] + n, width);
                self.cursor[0] = cmp::min(self.cursor[0] + n, width);
            }
            'D' => { // Cursor Backward
                let mut n = 1;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                self.writer[0] = self.writer[0].saturating_sub(n);
                self.cursor[0] = self.cursor[0].saturating_sub(n);
            }
            'G' => { // Cursor Horizontal Absolute
                let (_, y) = self.cursor_position();
                let mut x = 1;
                for param in params.iter() {
                    x = param[0] as usize; // 1-indexed value
                }
                if x == 0 || x > SCREEN_WIDTH {
                    return;
                }
                self.set_writer_position(x - 1, y);
                self.set_cursor_position(x - 1, y);
            }
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
                if x == 0 || y == 0 || x > SCREEN_WIDTH || y > SCREEN_HEIGHT {
                    return;
                }
                self.set_writer_position(x - 1, y - 1);
                self.set_cursor_position(x - 1, y - 1);
            }
            'J' => { // Erase in Display
                let mut n = 0;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                match n {
                    // TODO: 0 and 1, cursor to beginning or to end of screen
                    2 => self.clear_screen(),
                    _ => return,
                }
                self.set_writer_position(0, 0);
                self.set_cursor_position(0, 0);
            }
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
            }
            'h' => { // Enable
                for param in params.iter() {
                    match param[0] {
                        12 => self.enable_echo(),
                        25 => self.enable_cursor(),
                        _ => return,
                    }
                }
            }
            'l' => { // Disable
                for param in params.iter() {
                    match param[0] {
                        12 => self.disable_echo(),
                        25 => self.disable_cursor(),
                        _ => return,
                    }
                }
            }
            '~' => {
                for param in params.iter() {
                    match param[0] {
                        5 => self.scroll_up(SCREEN_HEIGHT),
                        6 => self.scroll_down(SCREEN_HEIGHT),
                        _ => continue,
                    }
                }
            }
            _ => {}
        }
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _: bool) {
        if params.len() == 1 {
            let s = String::from_utf8_lossy(params[0]);
            match s.chars().next() {
                Some('P') if s.len() == 8 => {
                    if let Ok((i, r, g, b)) = parse_palette(&s) {
                        self.set_palette(i, r, g, b);
                    }
                }
                Some('R') if s.len() == 1 => {
                    let palette = Palette::default();
                    for (i, (r, g, b)) in palette.colors.iter().enumerate() {
                        self.set_palette(i, *r, *g, *b);
                    }
                }
                _ => {}
            }
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

#[derive(Debug, Clone)]
pub struct VgaFont;

impl VgaFont {
    pub fn new() -> Self {
        Self
    }
}

impl FileIO for VgaFont {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, ()> {
        Err(()) // TODO
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        if let Ok(font) = Font::try_from(buf) {
            set_font(&font);
            Ok(buf.len()) // TODO: Use font.data.len() ?
        } else {
            Err(())
        }
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => false, // TODO
            IO::Write => true,
        }
    }
}

#[doc(hidden)]
pub fn print_fmt(args: fmt::Arguments) {
    interrupts::without_interrupts(||
        WRITER.lock().write_fmt(args).expect("Could not print to VGA")
    )
}

pub fn color() -> (Color, Color) {
    interrupts::without_interrupts(||
        WRITER.lock().color()
    )
}

pub fn set_color(foreground: Color, background: Color) {
    interrupts::without_interrupts(||
        WRITER.lock().set_color(foreground, background)
    )
}

// ASCII Printable
// Backspace
// New Line
// Carriage Return
// Extended ASCII Printable
pub fn is_printable(c: u8) -> bool {
    matches!(c, 0x20..=0x7E | 0x08 | 0x0A | 0x0D | 0x80..=0xFF)
}

// TODO: Remove this
pub fn set_font(font: &Font) {
    interrupts::without_interrupts(||
        WRITER.lock().set_font(font)
    )
}

// TODO: Remove this
pub fn set_palette(palette: Palette) {
    interrupts::without_interrupts(||
        for (i, (r, g, b)) in palette.colors.iter().enumerate() {
            WRITER.lock().set_palette(i, *r, *g, *b)
        }
    )
}
pub fn set_palette_color(i: usize, r: u8, g: u8, b: u8) {
    interrupts::without_interrupts(||
        WRITER.lock().set_palette(i, r, g, b)
    )
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
        let mut addr: Port<u8> = Port::new(ATTR_ADDR_REG);
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
        let mut addr: Port<u8> = Port::new(ATTR_ADDR_REG);
        let mut data: Port<u8> = Port::new(ATTR_READ_REG);
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

#[test_case]
fn test_parse_palette() {
    assert_eq!(parse_palette("P0282828"), Ok((0, 0x28, 0x28, 0x28)));
    assert_eq!(parse_palette("P4CC241D"), Ok((4, 0xCC, 0x24, 0x1D)));
    assert!(parse_palette("BAAAAAAD").is_ok());
    assert!(parse_palette("GOOOOOOD").is_err());
}
