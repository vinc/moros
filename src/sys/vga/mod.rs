mod color;
mod font;
mod buffer;
mod palette;
mod screen;
mod writer;

pub use font::VgaFont;
pub use screen::VgaMode;
pub use palette::Palette as VgaPalette;
pub use buffer::Buffer as VgaBuffer;

use color::Color;
use palette::Palette;
use writer::WRITER;

use crate::sys::x86::int;
use crate::sys::x86::port::*;

use alloc::string::String;
use bit_field::BitField;
use core::cmp;
use core::fmt;
use core::fmt::Write;
use core::num::ParseIntError;

const ATTR_ADDR_REG:           u16 = 0x3C0;
const ATTR_WRITE_REG:          u16 = 0x3C0;
const ATTR_READ_REG:           u16 = 0x3C1;
const MISC_WRITE_REG:          u16 = 0x3C2;
const SEQUENCER_ADDR_REG:      u16 = 0x3C4;
const SEQUENCER_DATA_REG:      u16 = 0x3C5;
const DAC_ADDR_READ_MODE_REG:  u16 = 0x3C7;
const DAC_ADDR_WRITE_MODE_REG: u16 = 0x3C8;
const DAC_DATA_REG:            u16 = 0x3C9;
const GRAPHICS_ADDR_REG:       u16 = 0x3CE;
const GRAPHICS_DATA_REG:       u16 = 0x3CF;
const CRTC_ADDR_REG:           u16 = 0x3D4;
const CRTC_DATA_REG:           u16 = 0x3D5;
const INPUT_STATUS_REG:        u16 = 0x3DA;
const INSTAT_READ_REG:         u16 = 0x3DA;

#[doc(hidden)]
pub fn print_fmt(args: fmt::Arguments) {
    int::without_interrupts(||
        WRITER.lock().write_fmt(args).expect("Could not print to VGA")
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

// 0x00 -> top
// 0x0F -> bottom
// 0x1F -> max (invisible)
fn set_underline_location(location: u8) {
    int::without_interrupts(|| {
        unsafe {
            outb(CRTC_ADDR_REG, 0x14); // Underline Location Register
            outb(CRTC_DATA_REG, location);
        }
    })
}

fn disable_underline() {
    set_underline_location(0x1F);
}

fn disable_blinking() {
    int::without_interrupts(|| {
        let reg = 0x10; // Attribute Mode Control Register
        let mut attr = get_attr_ctrl_reg(reg);
        attr.set_bit(3, false); // Clear "Blinking Enable" bit
        set_attr_ctrl_reg(reg, attr);
    })
}

fn set_attr_ctrl_reg(index: u8, value: u8) {
    int::without_interrupts(|| {
        unsafe {
            inb(INPUT_STATUS_REG); // Reset to address mode
            let tmp = inb(ATTR_ADDR_REG);
            outb(ATTR_ADDR_REG, index);
            outb(ATTR_ADDR_REG, value);
            outb(ATTR_ADDR_REG, tmp);
        }
    })
}

fn get_attr_ctrl_reg(index: u8) -> u8 {
    int::without_interrupts(|| {
        let index = index | 0x20; // Set "Palette Address Source" bit
        unsafe {
            inb(INPUT_STATUS_REG); // Reset to address mode
            let tmp = inb(ATTR_ADDR_REG);
            outb(ATTR_ADDR_REG, index);
            let res = inb(ATTR_READ_REG);
            outb(ATTR_ADDR_REG, tmp);
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

    screen::set_80x25_mode();
}
