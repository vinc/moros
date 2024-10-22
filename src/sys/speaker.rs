use super::clk;

use crate::api::fs::{FileIO, IO};

use core::convert::TryInto;
use x86_64::instructions::port::Port;

#[derive(Debug, Clone)]
pub struct Speaker;

impl Speaker {
    pub fn new() -> Self {
        Self {}
    }
}

impl FileIO for Speaker {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, ()> {
        Err(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        if let Ok(bytes) = buf.try_into() {
            match f64::from_be_bytes(bytes) {
                0.0 => stop_sound(),
                freq => start_sound(freq),
            }
            return Ok(8);
        }
        Err(())
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => false,
            IO::Write => true,
        }
    }
}

// See: https://wiki.osdev.org/PC_Speaker

const SPEAKER_PORT: u16 = 0x61;

fn start_sound(freq: f64) {
    debug!("speaker::start_sound({})", freq);
    let divider = (clk::pit_frequency() / freq) as u16;
    let channel = 2;
    clk::set_pit_frequency(divider, channel);

    let mut speaker: Port<u8> = Port::new(SPEAKER_PORT);
    let tmp = unsafe { speaker.read() };
    if tmp != (tmp | 3) {
        unsafe { speaker.write(tmp | 3) };
    }
}

fn stop_sound() {
    debug!("speaker::stop_sound()");
    let mut speaker: Port<u8> = Port::new(SPEAKER_PORT);
    let tmp = unsafe { speaker.read() } & 0xFC;
    unsafe { speaker.write(tmp) };
}
