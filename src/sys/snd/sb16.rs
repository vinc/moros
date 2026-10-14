use super::{SoundBuffer, SoundConfig};

use crate::sys;
use crate::sys::x86::port::*;

use alloc::collections::VecDeque;
use bit_field::BitField;

const TIMEOUT: u64 = 100_000_000; // 100 ms

// Sources:
// https://wiki.osdev.org/Sound_Blaster_16
// https://pdos.csail.mit.edu/6.828/2006/readings/hardware/SoundBlaster.pdf

const MIXER_ADDR: u16 = 0x224;
const MIXER_DATA: u16 = 0x225;
const DSP_RESET:  u16 = 0x226;
const DSP_READ:   u16 = 0x22A;
const DSP_WRITE:  u16 = 0x22C;
const DSP_ACK:    u16 = 0x22E;

const BLOCK_SIZE: usize = SoundBuffer::size();

struct DoubleBuffer {
    block: &'static mut [u8],
    index: usize
}

impl DoubleBuffer {
    fn new() -> Self {
        Self {
            block: sys::mem::dma::buffer(BLOCK_SIZE),
            index: 0
        }
    }

    fn addr(&self) -> usize {
        sys::mem::phys_addr(self.block.as_ptr())
    }

    fn reset(&mut self, buffer: &mut VecDeque<u8>) {
        self.index = 0;
        self.fill(0, buffer);
        self.fill(1, buffer);
    }

    fn swap(&mut self, buffer: &mut VecDeque<u8>) -> usize {
        let size = self.fill(self.index, buffer);
        self.index ^= 1;
        size
    }

    fn fill(&mut self, i: usize, buffer: &mut VecDeque<u8>) -> usize {
        let (left, right) = self.block.split_at_mut(BLOCK_SIZE / 2);
        let half = if i == 0 { left } else { right };
        let size = core::cmp::min(half.len(), buffer.len());
        for (dst, src) in half.iter_mut().zip(buffer.drain(..size)) {
            *dst = src;
        }
        half[size..].fill(0x80);
        size
    }

    fn clear(&mut self) {
        self.block.fill(0x80);
    }
}

pub struct Device {
    is_playing: bool,
    is_draining: bool,
    buffer: VecDeque<u8>,
    config: SoundConfig,
    blocks: DoubleBuffer,
}

impl Device {
    fn new() -> Self {
        init();
        Self {
            is_playing: false,
            is_draining: false,
            buffer: VecDeque::new(),
            config: SoundConfig::new(),
            blocks: DoubleBuffer::new(),
        }
    }

    pub fn play(&mut self, buffer: &[u8], config: &SoundConfig) {
        debug_assert!(config.channels == 1);
        debug_assert!(config.sample_bits == 8);
        debug_assert!(config.sample_rate <= u16::MAX as u32);
        self.config = config.clone();
        self.buffer.extend(buffer);
        if !self.is_playing {
            self.start();
        }
    }

    fn start(&mut self) {
        if self.buffer.len() < BLOCK_SIZE {
            // TODO: Handle very short audio files
            return;
        }
        self.is_playing = true;
        self.is_draining = false;
        self.blocks.reset(&mut self.buffer);

        // Program the DMA controller
        dma(self.blocks.addr(), BLOCK_SIZE - 1);

        // Set the DSP transfer sampling rate
        let rate = (self.config.sample_rate as u16).to_be_bytes();
        write_dsp(0x41); // Output
        write_dsp(rate[0]); // High byte
        write_dsp(rate[1]); // Low byte

        // Send an I/O command
        write_dsp(0xC6); // 8-bit output

        // Send the transfer mode
        write_dsp(0x00); // 8-bit mono unsigned PCM

        // Send the DSP block transfer size
        let bytes = ((BLOCK_SIZE / 2) - 1).to_le_bytes();
        write_dsp(bytes[0]);
        write_dsp(bytes[1]);
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        self.is_draining = false;
        write_dsp(0xD0); // Pause DMA mode
        let chan = 1;
        unsafe { outb(0x0A, 0x04 + chan) }; // Disable channel
        self.blocks.clear();
        self.buffer.clear();
        self.buffer.shrink_to_fit();
    }

    pub fn handle_interrupt(&mut self) {
        if self.is_draining && self.buffer.is_empty() {
            self.is_playing = false;
            self.is_draining = false;
            write_dsp(0xD0); // Pause DMA mode
            let chan = 1;
            unsafe { outb(0x0A, 0x04 + chan) }; // Disable channel
        } else {
            self.is_draining = self.blocks.swap(&mut self.buffer) == 0;
        }
        unsafe { inb(DSP_ACK) };
    }
}

// TODO: Return an error in case of timeout
fn write_dsp(val: u8) {
    let timer = sys::clk::timer(TIMEOUT);
    unsafe {
        while inb(DSP_WRITE).get_bit(7) && !timer.is_elapsed() {
            core::hint::spin_loop();
        }
        outb(DSP_WRITE, val);
    }
}

fn reset() -> bool {
    unsafe {
        outb(DSP_RESET, 1);
        sys::clk::wait(3000); // 3 microseconds
        outb(DSP_RESET, 0);

        let timer = sys::clk::timer(TIMEOUT);
        loop {
            if inb(DSP_ACK).get_bit(7) && inb(DSP_READ) == 0xAA {
                return true;
            }
            if timer.is_elapsed() {
                break;
            }
        }
        false
    }
}

fn dma(addr: usize, size: usize) {
    debug_assert_eq!(addr >> 24, 0); // Under 16 MB
    debug_assert_eq!(addr >> 16, (addr + size) >> 16); // Inside one 64 KB page
    let addr = addr.to_le_bytes();
    let size = size.to_le_bytes();
    let chan = 1;
    unsafe {
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
    write_dsp(0xD1); // Turn on speaker
    unsafe {
        outb(MIXER_ADDR, 0x80);
        outb(MIXER_DATA, irq(sys::pic::SND_IRQ));
    }
}

pub fn find() -> Option<Device> {
    if reset() {
        Some(Device::new())
    } else {
        None
    }
}
