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
const SPEAKER_CHANNEL: u8 = 0x02;
const SPEAKER_ENABLED: u8 = 0x03;
const SPEAKER_DISABLED: u8 = 0xFC;

fn start_sound(frequency: f64) {
    stop_sound();

    let divider = (clk::pit_frequency() / frequency) as u16;
    clk::set_pit_frequency(divider, SPEAKER_CHANNEL);

    let mut speaker: Port<u8> = Port::new(SPEAKER_PORT);
    let tmp = unsafe { speaker.read() };
    unsafe { speaker.write(tmp | SPEAKER_ENABLED) };
}

fn stop_sound() {
    let mut speaker: Port<u8> = Port::new(SPEAKER_PORT);
    let tmp = unsafe { speaker.read() };
    unsafe { speaker.write(tmp & SPEAKER_DISABLED) };
}
