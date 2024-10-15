use super::*;

use crate::api::fs::{FileIO, IO};

use core::convert::TryFrom;
use spin::Mutex;

static PALETTE: Mutex<Option<Palette>> = Mutex::new(None);

const DEFAULT_COLORS: [(u8, u8, u8); 16] = [
    (0x00, 0x00, 0x00), // DarkBlack
    (0x00, 0x00, 0x80), // DarkBlue
    (0x00, 0x80, 0x00), // DarkGreen
    (0x00, 0x80, 0x80), // DarkCyan
    (0x80, 0x00, 0x00), // DarkRed
    (0x80, 0x00, 0x80), // DarkMagenta
    (0x80, 0x80, 0x00), // DarkYellow
    (0xC0, 0xC0, 0xC0), // DarkWhite
    (0x80, 0x80, 0x80), // BrightBlack
    (0x00, 0x00, 0xFF), // BrightBlue
    (0x00, 0xFF, 0x00), // BrightGreen
    (0x00, 0xFF, 0xFF), // BrightCyan
    (0xFF, 0x00, 0x00), // BrightRed
    (0xFF, 0x00, 0xFF), // BrightMagenta
    (0xFF, 0xFF, 0x00), // BrightYellow
    (0xFF, 0xFF, 0xFF), // BrightWhite
];

#[derive(Debug, Clone)]
pub struct Palette {
    pub colors: [(u8, u8, u8); 256],
}

impl Palette {
    pub fn new() -> Self {
        Self { colors: [(0, 0, 0); 256] }
    }

    pub fn default() -> Self {
        let mut palette = Palette::new();
        for (i, (r, g, b)) in DEFAULT_COLORS.iter().enumerate() {
            let i = Color::from_index(i).register();
            palette.colors[i] = (*r, *g, *b);
        }
        palette
    }

    pub fn read() -> Self {
        let mut palette = Palette::new();
        for i in 0..256 {
            palette.colors[i] = read_palette(i);
        }
        palette
    }

    pub fn write(&self) {
        for (i, (r, g, b)) in self.colors.iter().enumerate() {
            write_palette(i, *r, *g, *b);
        }
    }

    pub fn to_bytes(&self) -> [u8; 256 * 3] {
        let mut buf = [0; 256 * 3];
        for (i, (r, g, b)) in self.colors.iter().enumerate() {
            buf[i * 3 + 0] = *r;
            buf[i * 3 + 1] = *g;
            buf[i * 3 + 2] = *b;
        }
        buf
    }

    pub fn size() -> usize {
        256 * 3
    }
}

impl TryFrom<&[u8]> for Palette {
    type Error = ();

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        if buf.len() != Palette::size() {
            return Err(());
        }
        let mut colors = [(0, 0, 0); 256];
        for (i, rgb) in buf.chunks(3).enumerate() {
            colors[i] = (rgb[0], rgb[1], rgb[2])
        }

        Ok(Palette { colors })
    }
}

impl FileIO for VgaPalette {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let res = Palette::read().to_bytes();
        if buf.len() < res.len() {
            return Err(());
        }
        buf.clone_from_slice(&res);
        Ok(res.len())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        let palette = Palette::try_from(buf)?;
        palette.write();
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

fn write_palette(i: usize, r: u8, g: u8, b: u8) {
    interrupts::without_interrupts(||
        WRITER.lock().set_palette(i, r, g, b)
    )
}

fn read_palette(i: usize) -> (u8, u8, u8) {
    interrupts::without_interrupts(||
        WRITER.lock().palette(i)
    )
}

pub fn restore_palette() {
    if let Some(ref palette) = *PALETTE.lock() {
        palette.write();
    }
}

pub fn backup_palette() {
    *PALETTE.lock() = Some(Palette::read())
}
