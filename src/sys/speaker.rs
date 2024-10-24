use super::clk;

use crate::api::fs::{FileIO, IO};

use alloc::string::String;
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
        if let Ok(s) = String::from_utf8(buf.to_vec()) {
            if let Ok(n) = s.parse() {
                if n > 0.0 {
                    start_sound(n);
                } else {
                    stop_sound();
                }
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
    let divider = (clk::pit_frequency() / freq) as u16;
    let channel = 2; // PC Speaker
    clk::set_pit_frequency(divider, channel);

    let mut speaker: Port<u8> = Port::new(SPEAKER_PORT);
    let tmp = unsafe { speaker.read() };
    if tmp != (tmp | 3) {
        unsafe { speaker.write(tmp | 3) };
    }
}

fn stop_sound() {
    let mut speaker: Port<u8> = Port::new(SPEAKER_PORT);
    let tmp = unsafe { speaker.read() } & 0xFC;
    unsafe { speaker.write(tmp) };
}
