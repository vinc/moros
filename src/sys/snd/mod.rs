mod sb16;

use crate::api::fs::{FileIO, IO};
use core::convert::TryFrom;
use core::convert::TryInto;

struct SndConfig {
    channels: u16,
    sample_bits: u16,
    sample_rate: u32,
    data_start: u32,
    data_end: u32,
}

impl SndConfig {
    pub fn new(buf: &[u8]) -> Self {
        Self {
            channels: 1,
            sample_bits: 8,
            sample_rate: 44100,
            data_start: 0,
            data_end: buf.len() as u32,
        }
    }
}

impl TryFrom<&[u8]> for SndConfig {
    type Error = ();

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        if buf.len() < 44 {
            debug!("SND: Error buf size");
            return Err(());
        }
        if buf[0..4] != *b"RIFF" {
            debug!("SND: Error parsing 'RIFF'");
            return Err(());
        }
        if buf[8..12] != *b"WAVE" {
            debug!("SND: Error parsing 'WAVE'");
            return Err(());
        }
        if buf[12..16] != *b"fmt " {
            debug!("SND: Error parsing 'fmt '");
            return Err(());
        }
        if buf[20..22] != 1u16.to_le_bytes() { // Audio format
            debug!("SND: Error parsing audio format");
            return Err(());
        }

        let channels = u16::from_le_bytes(
            buf[22..24].try_into().map_err(|_| ())?
        );
        let sample_rate = u32::from_le_bytes(
            buf[24..28].try_into().map_err(|_| ())?
        );
        let sample_bits = u16::from_le_bytes(
            buf[34..36].try_into().map_err(|_| ())?
        );
        if buf[36..40] != *b"data" {
            debug!("SND: Error parsing 'data'");
            return Err(());
        }
        let data_start = 44;
        let data_end = data_start + u32::from_le_bytes(
            buf[40..44].try_into().map_err(|_| ())?
        );

        Ok(SndConfig {
            channels,
            sample_bits,
            sample_rate,
            data_start,
            data_end,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SndBuffer;

impl SndBuffer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn size() -> usize {
        sb16::BUF_LEN
    }
}

impl FileIO for SndBuffer {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, ()> {
        Err(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        if buf.is_empty() {
            sb16::stop();
        } else {
            let config = if buf.get(0..4) == Some(b"RIFF") {
                SndConfig::try_from(buf)?
            } else {
                SndConfig::new(&buf)
            };
            let start = config.data_start as usize;
            let end = config.data_end as usize;
            sb16::play(&buf[start..end], &config);
        }
        Ok(buf.len())
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => false,
            IO::Write => true,
        }
    }
}

pub fn init() {
    sb16::init();
}
