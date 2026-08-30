use super::*;

use buffer::Buffer;

#[cfg(target_arch = "x86_64")] // TODO: Remove
use crate::sys::fs::{FileIO, IO};

use spin::Mutex;

#[derive(Copy, Clone)]
enum ModeName { // TODO: Rename to Resolution
    C80x25,
    P320x200x256,
    P640x480x16,
}

static MODE: Mutex<Option<ModeName>> = Mutex::new(None);

// Source: https://www.singlix.com/trdos/archive/vga/Graphics%20in%20pmode.pdf
const C_80_25: [u8; 61] = [
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

const P_320_200_256: [u8; 61] = [
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

const P_640_480_16: [u8; 61] = [
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

// Source: https://www.singlix.com/trdos/archive/vga/Graphics%20in%20pmode.pdf
fn set_mode(mode: ModeName) {
    *MODE.lock() = Some(mode);
    let mut regs = match mode {
        ModeName::C80x25 => C_80_25,
        ModeName::P320x200x256 => P_320_200_256,
        ModeName::P640x480x16 => P_640_480_16,
    };

    int::without_interrupts(|| {
        let mut i = 0;

        unsafe {
            outb(MISC_WRITE_REG, regs[i]);
            i += 1;

            for j in 0..SEQ_REGS_COUNT {
                outb(SEQUENCER_ADDR_REG, j as u8);
                outb(SEQUENCER_DATA_REG, regs[i]);
                i += 1;
            }

            // Unlock CRTC regs
            outb(CRTC_ADDR_REG, 0x03);
            let data = inb(CRTC_DATA_REG);
            outb(CRTC_DATA_REG, data | 0x80);
            outb(CRTC_ADDR_REG, 0x11);
            let data = inb(CRTC_DATA_REG);
            outb(CRTC_DATA_REG, data & !0x80);

            // Keep them unlocked
            regs[0x03] |= 0x80;
            regs[0x11] &= !0x80;

            for j in 0..CRTC_REGS_COUNT {
                outb(CRTC_ADDR_REG, j as u8);
                outb(CRTC_DATA_REG, regs[i]);
                i += 1;
            }

            for j in 0..GC_REGS_COUNT {
                outb(GRAPHICS_ADDR_REG, j as u8);
                outb(GRAPHICS_DATA_REG, regs[i]);
                i += 1;
            }

            for j in 0..AC_REGS_COUNT {
                inb(INSTAT_READ_REG);
                outb(ATTR_ADDR_REG, j as u8);
                outb(ATTR_WRITE_REG, regs[i]);
                i += 1;
            }

            // Lock 16-color palette and unblank display
            inb(INSTAT_READ_REG);
            outb(ATTR_ADDR_REG, 0x20);
        }
    });
}

fn is_80x25c_mode() -> bool {
    match *MODE.lock() {
        Some(ModeName::C80x25) | None => true,
        _ => false
    }
}

fn set_80x25c_mode() {
    let restorable = MODE.lock().is_some();
    clear_screen();
    set_mode(ModeName::C80x25);
    disable_blinking();
    disable_underline();
    if restorable {
        palette::restore_palette();

        #[cfg(target_arch = "x86_64")] // TODO: Remove
        font::restore_font();
    }
}

fn set_320x200p_mode() {
    if is_80x25c_mode() {
        palette::backup_palette();
    }
    set_mode(ModeName::P320x200x256);
    clear_screen();
}

fn set_640x480p_mode() {
    if is_80x25c_mode() {
        palette::backup_palette();
    }
    set_mode(ModeName::P640x480x16);
    clear_screen();
}

fn clear_screen() {
    let size = match *MODE.lock() {
        Some(ModeName::P320x200x256) => 320 * 200,
        Some(ModeName::P640x480x16) => (640 / 4 / 2) * 480,
        _ => return,
    };
    // FIXME: This only work for 320x200 linear buffer
    let dst = Buffer::addr() as *mut u8;
    unsafe {
        core::ptr::write_bytes(dst, 0, size);
    }
}

#[derive(Debug, Clone)]
pub struct VgaMode; // TODO: Rename to VgaResolution

impl VgaMode {
    pub fn new() -> Self {
        Self
    }

    pub fn size() -> usize {
        // Must be at least 4 + 1 + 4 + 1 bytes: "<width>x<height><mode>"
        16
    }
}

#[cfg(target_arch = "x86_64")] // TODO: Remove
impl FileIO for VgaMode {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        match *MODE.lock() {
            Some(ModeName::C80x25) | None => write_mode(buf, b"80x25c"),
            Some(ModeName::P320x200x256) => write_mode(buf, b"320x200p"),
            Some(ModeName::P640x480x16) => write_mode(buf, b"640x480p"),
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        match buf {
            b"80x25c" => set_80x25c_mode(),
            b"320x200p" => set_320x200p_mode(),
            b"640x480p" => set_640x480p_mode(),
            _ => return Err(()),
        }
        Ok(buf.len())
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => true,
            IO::Write => true,
        }
    }
}

fn write_mode(buf: &mut [u8], mode: &[u8]) -> Result<usize, ()> {
    let n = mode.len();
    if buf.len() < n {
        Err(())
    } else {
        buf[0..n].clone_from_slice(mode);
        Ok(n)
    }
}

pub fn set_text_mode() {
    set_80x25c_mode();
}
