use super::writer::WRITER;

use crate::api::font::Font;
use crate::api::fs::{FileIO, IO};

use core::convert::TryFrom;
use x86_64::instructions::interrupts;

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

// TODO: Remove this
pub fn set_font(font: &Font) {
    interrupts::without_interrupts(||
        WRITER.lock().set_font(font)
    )
}
