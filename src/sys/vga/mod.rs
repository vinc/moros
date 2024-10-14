mod font;
mod screen;
mod writer;

pub use font::VgaFont;
pub use screen::VgaMode;
use writer::WRITER;

use crate::api::vga::color;
use crate::api::vga::{Color, Palette};

use alloc::string::String;
use bit_field::BitField;
use core::cmp;
use core::fmt;
use core::fmt::Write;
use core::num::ParseIntError;
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

// Convert 8-bit to 6-bit color
fn vga_color(color: u8) -> u8 {
    color >> 2
}

// TODO: Remove this
fn parse_palette(palette: &str) -> Result<(usize, u8, u8, u8), ParseIntError> {
    debug_assert!(palette.len() == 8);
    debug_assert!(palette.starts_with('P'));

    let i = usize::from_str_radix(&palette[1..2], 16)?;
    let r = u8::from_str_radix(&palette[2..4], 16)?;
    let g = u8::from_str_radix(&palette[4..6], 16)?;
    let b = u8::from_str_radix(&palette[6..8], 16)?;

    Ok((i, r, g, b))
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
pub fn set_palette(palette: Palette) {
    interrupts::without_interrupts(||
        for (i, (r, g, b)) in palette.colors.iter().enumerate() {
            let i = color::from_index(i).to_vga_reg() as usize;
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

fn disable_underline() {
    set_underline_location(0x1F);
}

fn disable_blinking() {
    interrupts::without_interrupts(|| {
        let reg = 0x10; // Attribute Mode Control Register
        let mut attr = get_attr_ctrl_reg(reg);
        attr.set_bit(3, false); // Clear "Blinking Enable" bit
        set_attr_ctrl_reg(reg, attr);
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

    disable_blinking();
    disable_underline();

    WRITER.lock().clear_screen();
}

#[test_case]
fn test_parse_palette() {
    assert_eq!(parse_palette("P0282828"), Ok((0, 0x28, 0x28, 0x28)));
    assert_eq!(parse_palette("P4CC241D"), Ok((4, 0xCC, 0x24, 0x1D)));
    assert!(parse_palette("BAAAAAAD").is_ok());
    assert!(parse_palette("GOOOOOOD").is_err());
}
