use super::{SoundBuffer, SoundConfig};

use crate::sys;
use crate::sys::port::*;
use crate::sys::mem::PhysBuf;

use alloc::vec::Vec;
use alloc::sync::Arc;
use core::ptr;
use spin::Mutex;

// Sources:
// https://wiki.osdev.org/AC97

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, align(16))]
struct BufDesc {
    addr: u32,
    size: u16,
    ctrl: u16
}

pub struct Device {
    is_playing: bool,
    buffer: Vec<u8>,
    config: SoundConfig,
    block: PhysBuf,
    bar0: u16,
    bar1: u16,
    bdl: Arc<Mutex<[BufDesc; 32]>>, // Buffer Descriptor List
}

impl Device {
    pub fn new(bar0: u16, bar1: u16) -> Self {
        debug!("new()");
        outl(bar1 + 0x2C, 3); // Global Control Register
        sys::clk::wait(100_000); // 100ms
        outw(bar0 + 0x00, 1); // Reset Register
        sys::clk::wait(100_000); // 100ms
        println!("SND AC97 RC: {:#016b}", inw(bar0 + 0x00)); // Capabilities
        println!("SND AC97 EC: {:#016b}", inw(bar0 + 0x28)); // Ext Cap
        println!("SND AC97 GS: {:#032b}", inl(bar1 + 0x30)); // Global Status

        Self {
            is_playing: false,
            buffer: Vec::new(),
            config: SoundConfig::new(),
            block: PhysBuf::new(SoundBuffer::size()),
            bdl: Arc::new(Mutex::new([(); 32].map(|_| BufDesc::default()))),
            bar0, bar1
        }
    }

    pub fn play(&mut self, buffer: &[u8], config: &SoundConfig) {
        debug!("play(buf.len={}, {:?})", buffer.len(), config);
        println!("SND AC97 Output Channel: {:#08b}", inb(self.bar1 + 0x1B));
        outb(self.bar1 + 0x1B, 0x02); // Set reset bit of output channel
        while inb(self.bar1 + 0x1B) & 0x02 != 0 {
            // Wait for reset to be completed
        }
        println!("SND AC97 Output Channel: {:#08b}", inb(self.bar1 + 0x1B));

        outw(self.bar0 + 0x2A, 1);
        outw(self.bar0 + 0x2C, config.sample_rate as u16);
        println!("SND AC97 Sample Rate: {} Hz", inw(self.bar0 + 0x2C));

        // Write physical position of BDL to Buffer Descriptor Base Address
        // register
        let mut bdl = self.bdl.lock();

        let n = core::cmp::min(self.block.len(), buffer.len());
        self.block[0..n].copy_from_slice(&buffer[0..n]);

        bdl[0].addr = self.block.addr() as u32;
        bdl[0].size = (n / 4) as u16;
        bdl[0].ctrl = 1 << 15; // IOC: Interrupt on Completion

        let ptr = ptr::addr_of!(bdl[0]) as *const u8;
        let addr = sys::mem::phys_addr(ptr);
        outl(self.bar1 + 0x10, addr as u32);

        // Write number of last valid buffer entry to Last Valid Entry register
        outb(self.bar1 + 0x15, 0);

        // Start DMA
        outb(self.bar1 + 0x1B, 0x01);

        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        debug!("stop()");
        if self.is_playing {
            // Stop DMA
            outb(self.bar1 + 0x1B, 0);
            self.is_playing = false;
        }
    }

    pub fn handle_interrupt(&mut self) {
        debug!("handle_interrupt()");
        // Clear channel status registers
        outw(self.bar1 + 0x06, 0x1C);
        outw(self.bar1 + 0x16, 0x1C);
        outw(self.bar1 + 0x26, 0x1C);
    }
}
