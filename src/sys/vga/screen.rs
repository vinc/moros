use super::*;

use crate::api::fs::{FileIO, IO};

use spin::Mutex;

#[derive(Copy, Clone)]
enum ModeName {
    T80x25,
    G320x200x256,
    G640x480x16,
}

static MODE: Mutex<Option<ModeName>> = Mutex::new(None);

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

// Source: https://www.singlix.com/trdos/archive/vga/Graphics%20in%20pmode.pdf
fn set_mode(mode: ModeName) {
    *MODE.lock() = Some(mode);
    let mut regs = match mode {
        ModeName::T80x25 => T_80_25,
        ModeName::G320x200x256 => G_320_200_256,
        ModeName::G640x480x16 => G_640_480_16,
    }.to_vec();

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

fn is_80x25_mode() -> bool {
    match *MODE.lock() {
        Some(ModeName::T80x25) | None => true,
        _ => false
    }
}

fn set_80x25_mode() {
    set_mode(ModeName::T80x25);
    disable_blinking();
    disable_underline();
    palette::restore_palette();
    font::restore_font();
}

fn set_320x200_mode() {
    if is_80x25_mode() {
        palette::backup_palette();
    }
    set_mode(ModeName::G320x200x256);
    // TODO: Clear screen
}

fn set_640x480_mode() {
    if is_80x25_mode() {
        palette::backup_palette();
    }
    set_mode(ModeName::G640x480x16);
    // TODO: Clear screen
}

#[derive(Debug, Clone)]
pub struct VgaMode;

impl VgaMode {
    pub fn new() -> Self {
        Self
    }
}

impl FileIO for VgaMode {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, ()> {
        Err(()) // TODO
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        match buf {
            b"80x25" => set_80x25_mode(),
            b"320x200" => set_320x200_mode(),
            b"640x480" => set_640x480_mode(),
            _ => return Err(()),
        }
        Ok(buf.len())
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => false, // TODO
            IO::Write => true,
        }
    }
}
