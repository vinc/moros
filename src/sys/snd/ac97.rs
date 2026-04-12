use super::{SoundBuffer, SoundConfig};

use crate::sys;
use crate::sys::port::*;
use crate::sys::mem::PhysBuf;

use alloc::vec::Vec;
use alloc::sync::Arc;
use bit_field::BitField;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

// Sources:
// https://wiki.osdev.org/AC97

const BDL: usize = 32;

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
struct BufDesc {
    addr: u32,
    size: u16,
    ctrl: u16
}

pub struct Device {
    is_playing: bool,
    config: SoundConfig,
    buffer: Vec<u8>,
    blocks: [PhysBuf; BDL],
    index: Arc<AtomicUsize>,
    bdl: Arc<Mutex<[BufDesc; BDL]>>, // Buffer Descriptor List
    bar0: u16,
    bar1: u16,
}

impl Device {
    pub fn new(bar0: u16, bar1: u16) -> Self {
        debug!("new()");
        Self {
            is_playing: false,
            config: SoundConfig::new(),
            buffer: Vec::new(),
            blocks: [(); BDL].map(|_| PhysBuf::new(SoundBuffer::size())),
            index: Arc::new(AtomicUsize::new(0)),
            bdl: Arc::new(Mutex::new([(); BDL].map(|_| BufDesc::default()))),
            bar0, bar1
        }
    }

    pub fn init(&mut self) {

        outl(self.bar1 + 0x2C, 0x02); // Cold reset
        sys::clk::wait(100_000); // 100ms
        outw(self.bar0 + 0x00, 1); // Reset Register
        sys::clk::wait(100_000); // 100ms
        debug!("SND AC97 RC: {:#016b}", inw(self.bar0 + 0x00)); // Capabilities
        debug!("SND AC97 EC: {:#016b}", inw(self.bar0 + 0x28)); // Ext Cap
        debug!("SND AC97 GS: {:#032b}", inl(self.bar1 + 0x30)); // Global Status

        /*
        // Setup BDL
        let mut bdl = self.bdl.lock();
        for i in 0..BDL {
            self.blocks[i].fill(0x00);
            bdl[i].addr = self.blocks[i].addr() as u32;
            bdl[i].size = 0;
            bdl[i].ctrl = 1 << 15; // IOC: Interrupt on Completion
        }

        // Write BDL addr to Buffer Descriptor Base Address register
        let addr = sys::mem::phys_addr(bdl.as_ptr() as *const u8);
        outl(self.bar1 + 0x10, addr as u32);
        */
    }

    fn fill_next_block(&mut self) -> usize {
        let mut bdl = self.bdl.lock();

        for j in 0..BDL {
            //debug!("SND AC97 BDL[{}].ctrl = {:#016b}", j, bdl[j].ctrl);
        }

        let i = self.index.update(Ordering::SeqCst, Ordering::SeqCst, |i| {
            (i + 1) % BDL
        });
        //debug!("SND AC97 bdl[{:02}]", i);

        let n = core::cmp::min(self.blocks[i].len(), self.buffer.len());
        self.blocks[i][0..n].copy_from_slice(&self.buffer[0..n]);
        self.blocks[i][n..].fill(0x00);
        self.buffer.drain(0..n);

        bdl[i].addr = self.blocks[i].addr() as u32;
        bdl[i].size = (n / 2) as u16;
        bdl[i].ctrl = 1 << 15; // IOC: Interrupt on Completion

        i
    }

    pub fn play(&mut self, buffer: &[u8], config: &SoundConfig) {
        self.buffer.extend_from_slice(buffer);

        if self.is_playing {
            return;
        }

        if self.buffer.len() < self.blocks[0].len() {
            return;
        }

        // Clear status
        //outw(self.bar1 + 0x16, 0x1C);
        //outl(self.bar1 + 0x30, inl(self.bar1 + 0x30));

        // Set reset bit of output channel
        //debug!("SND AC97 Output Channel: {:#08b}", inb(self.bar1 + 0x1B));
        outb(self.bar1 + 0x1B, 0x02);
        while inb(self.bar1 + 0x1B) & 0x02 != 0 {
            // Wait for reset to be completed
        }
        //debug!("SND AC97 Output Channel: {:#08b}", inb(self.bar1 + 0x1B));

        // Set sample rate
        //debug_assert!(inw(self.bar0 + 0x28).get_bit(0));
        outw(self.bar0 + 0x2A, 1);
        outw(self.bar0 + 0x2C, config.sample_rate as u16);
        //debug!("SND AC97 Sample Rate: {} Hz", inw(self.bar0 + 0x2C));

        // Write BDL addr to Buffer Descriptor Base Address register
        let mut bdl = self.bdl.lock();
        let addr = sys::mem::phys_addr(bdl.as_ptr() as *const u8);
        outl(self.bar1 + 0x10, addr as u32);
        drop(bdl);

        // Load sound data to memory
        let n = buffer.len() / self.blocks[0].len();
        for i in 0..n {
            self.fill_next_block();
        }
        let i = self.fill_next_block();

        // Write BDL index to Last Valid Entry register
        outb(self.bar1 + 0x15, i as u8);

        // Start DMA
        // outb(self.bar1 + 0x1B, 0x01);
        outb(self.bar1 + 0x1B, 0x01 | 0x08); // With interrupts

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
        outw(self.bar1 + 0x16, 0x1C);

        if self.buffer.is_empty() {
            self.stop();
        } else {
            let i = self.fill_next_block();
            outb(self.bar1 + 0x15, i as u8);
        }
    }
}
