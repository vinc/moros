use super::{SoundBuffer, SoundConfig};

use crate::sys;
use crate::sys::port::*;
use crate::sys::mem::PhysBuf;

use alloc::vec::Vec;

// Sources:
// https://wiki.osdev.org/Sound_Blaster_16
// https://pdos.csail.mit.edu/6.828/2006/readings/hardware/SoundBlaster.pdf

pub const IRQ: u8 = 5;

const MIXER_ADDR: u16 = 0x224;
const MIXER_DATA: u16 = 0x225;
const DSP_RESET:  u16 = 0x226;
const DSP_READ:   u16 = 0x22A;
const DSP_WRITE:  u16 = 0x22C;
const DSP_ACK:    u16 = 0x22E;

pub struct Device {
    is_playing: bool,
    buffer: Vec<u8>,
    config: SoundConfig,
    block: PhysBuf,
}

impl Device {
    fn new() -> Self {
        init();
        Self {
            is_playing: false,
            buffer: Vec::new(),
            config: SoundConfig::new(),
            block: PhysBuf::new(SoundBuffer::size()),
        }
    }

    pub fn play(&mut self, buffer: &[u8], config: &SoundConfig) {
        debug_assert!(config.channels == 1);
        debug_assert!(config.sample_bits == 8);
        debug_assert!(config.sample_rate <= u16::MAX as u32);
        self.config = config.clone();
        self.buffer.extend_from_slice(buffer);
        if !self.is_playing {
            self.start();
        }
    }

    fn start(&mut self) {
        if self.buffer.len() < self.block.len() {
            // TODO: Handle very short audio files
            return;
        }
        self.is_playing = true;
        self.fill_block();

        // Program the DMA controller
        dma(self.block.addr(), self.block.size() - 1);

        // Set the DSP transfer sampling rate
        let rate = (self.config.sample_rate as u16).to_be_bytes();
        outb(DSP_WRITE, 0x41); // Output
        outb(DSP_WRITE, rate[0]); // High byte
        outb(DSP_WRITE, rate[1]); // Low byte

        // Send an I/O command
        outb(DSP_WRITE, 0xC6); // 8-bit output

        // Send the transfer mode
        outb(DSP_WRITE, 0x00); // 8-bit mono unsigned PCM

        // Send the DSP block transfer size
        let bytes = (self.block.size() - 1).to_le_bytes();
        outb(DSP_WRITE, bytes[0]);
        outb(DSP_WRITE, bytes[1]);
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        outb(DSP_WRITE, 0xD0); // Pause DMA playback
        let chan = 1;
        outb(0x0A, 0x04 + chan); // Disable channel
        self.block.fill(0x80);
        self.buffer.clear();
        self.buffer.shrink_to_fit();
    }

    pub fn handle_interrupt(&mut self) {
        if self.buffer.is_empty() {
            self.is_playing = false;
            outb(DSP_WRITE, 0xD0); // Pause
            let chan = 1;
            outb(0x0A, 0x04 + chan); // Disable channel
        } else {
            self.fill_block();
        }
        let _ = inb(DSP_ACK);
    }

    fn fill_block(&mut self) {
        let len = core::cmp::min(self.block.len(), self.buffer.len());
        self.block[0..len].copy_from_slice(&self.buffer[0..len]);
        self.block[len..].fill(0x80);
        self.buffer.drain(0..len);
    }
}

fn reset() -> bool {
    outb(DSP_RESET, 1);
    sys::clk::wait(3000); // 3 microseconds
    outb(DSP_RESET, 0);
    for _ in 0..100 {
        sys::clk::wait(1000);
        if inb(DSP_READ) == 0xAA {
            return true;
        }
    }
    false
}

fn dma(addr: u64, size: usize) {
    let addr = addr.to_le_bytes();
    let size = size.to_le_bytes();
    let chan = 1;
    outb(0x0A, 0x04 + chan); // Disable channel
    outb(0x0C, 0x01);        // Flip flop
    outb(0x0B, 0x58 + chan); // Send transfer mode
    outb(0x83, addr[2]);     // Send page number
    outb(0x02, addr[0]);     // Send low bits of addr
    outb(0x02, addr[1]);     // Send high bits of addr
    outb(0x03, size[0]);     // Send low bits of size
    outb(0x03, size[1]);     // Send high bits of size
    outb(0x0A, chan);        // Enable channel
}

fn irq(num: u8) -> u8 {
    match num {
        2 => 0x01,
        5 => 0x02,
        7 => 0x04,
        10 => 0x08,
        _ => panic!(),
    }
}

pub fn init() {
    outb(MIXER_ADDR, 0x80);
    outb(MIXER_DATA, irq(IRQ));
}

pub fn find() -> Option<Device> {
    if reset() {
        Some(Device::new())
    } else {
        None
    }
}
