use super::{SoundBuffer, SoundConfig};

use crate::sys;
use crate::sys::port::*;
use crate::sys::mem::PhysBuf;

use alloc::vec::Vec;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

// Sources:
// https://wiki.osdev.org/AC97

// Native Audio Bus Master Control Registers
const PO_BDBAR: u16 = 0x10; // PCM Out Buffer Descriptor list Base Address
const PO_CIV:   u16 = 0x14; // PCM Out Current Index Value
const PO_LVI:   u16 = 0x15; // PCM Out Last Valid Index
const PO_SR:    u16 = 0x16; // PCM Out Status
const PO_CR:    u16 = 0x1B; // PCM Out Control
const GLOB_CNT: u16 = 0x2C; // Global Control

// Control Register
const RPBM:  u8 = 1 << 0; // Run/Pause Bus Master
const RR:    u8 = 1 << 1; // Reset Registers
const LVBIE: u8 = 1 << 2; // Last Valid Buffer Interrupt Enable
const IOCE:  u8 = 1 << 4; // Interrupt on Completion Enable

const BDL: usize = 32;

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, align(8))]
struct BufDesc {
    addr: u32,
    size: u16,
    ctrl: u16
}

pub struct Device {
    is_playing: bool,
    //config: SoundConfig,
    buffer: Vec<u8>,
    blocks: [PhysBuf; BDL],
    index: Arc<AtomicUsize>,
    bdl: Arc<Mutex<[BufDesc; BDL]>>, // Buffer Descriptor List
    bar0: u16,
    bar1: u16,
}

impl Device {
    pub fn new(bar0: u16, bar1: u16) -> Self {
        Self {
            is_playing: false,
            //config: SoundConfig::new(),
            buffer: Vec::new(),
            blocks: [(); BDL].map(|_| PhysBuf::new(SoundBuffer::size())),
            index: Arc::new(AtomicUsize::new(0)),
            bdl: Arc::new(Mutex::new([(); BDL].map(|_| BufDesc::default()))),
            bar0, bar1
        }
    }

    pub fn init(&mut self) {
        outl(self.bar1 + GLOB_CNT, 0x02); // Cold reset
        sys::clk::wait(100_000);
        outw(self.bar0 + 0x00, 1); // Reset Register
        sys::clk::wait(100_000);
        //debug!("SND AC97 RC: {:#016b}", inw(self.bar0 + 0x00)); // Capabilities
        //debug!("SND AC97 EC: {:#016b}", inw(self.bar0 + 0x28)); // Ext Cap
        //debug!("SND AC97 GS: {:#032b}", inl(self.bar1 + 0x30)); // Global Status

        let mut global_control = inl(self.bar1 + GLOB_CNT);
        global_control |= 0x01; // Set Global Interrupt Enable
        global_control |= 0x10; // PCM Out Interrupt Enable (PINTEN)
        outl(self.bar1 + GLOB_CNT, global_control);
    }

    fn fill_next_block(&mut self) -> usize {
        let mut bdl = self.bdl.lock();
        let i = self.index.update(Ordering::SeqCst, Ordering::SeqCst, |i| {
            (i + 1) % BDL
        });

        /*
        for j in 0..BDL {
            debug!("SND AC97 BDL[{}].ctrl = {:#016b}", j, bdl[j].ctrl);
        }
        debug!("SND AC97 bdl[{:02}]", i);
        debug!("  PO_CIV: {:02}", inb(self.bar1 + PO_CIV));
        debug!("  PO_LVI: {:02}", inb(self.bar1 + PO_LVI));
        */

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

        // Set reset bit of output channel
        outb(self.bar1 + PO_CR, RR);
        while inb(self.bar1 + PO_CR) & RR != 0 {
            // Wait for reset to be completed
            core::hint::spin_loop();
        }
        self.index.store(0, Ordering::SeqCst);

        // Set sample rate
        //debug!("SND AC97 Ext Cap: {:#016b}", inw(self.bar0 + 0x28));
        //debug!("SND AC97 Sample Rate: {} Hz", inw(self.bar0 + 0x2C));
        debug_assert_ne!(inw(self.bar0 + 0x28) & 0x01, 0);
        outw(self.bar0 + 0x2A, 1);
        outw(self.bar0 + 0x2C, config.sample_rate as u16);
        //debug!("SND AC97 Sample Rate: {} Hz", inw(self.bar0 + 0x2C));

        // Write BDL address to Buffer Descriptor Base Address register
        let bdl = self.bdl.lock();
        let addr = sys::mem::phys_addr(bdl.as_ptr() as *const u8);
        outl(self.bar1 + PO_BDBAR, addr as u32);
        drop(bdl);

        // Load sound data to memory
        let index = self.fill_next_block();
        debug_assert_eq!(index, 0);

        // Write BDL index to Last Valid Entry register
        outb(self.bar1 + PO_LVI, index as u8);

        // Clear any pending status bits before starting
        outw(self.bar1 + PO_SR, 0x1C);

        // Start DMA with interrupts
        outb(self.bar1 + PO_CR, RPBM | LVBIE | IOCE);

        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        if self.is_playing {
            outb(self.bar1 + PO_CR, 0); // Stop DMA
            self.is_playing = false;
            for i in 0..BDL {
                self.blocks[i].fill(0x00);
            }
        }
    }

    pub fn handle_interrupt(&mut self) {
        // Clear channel status registers
        outw(self.bar1 + PO_SR, 0x1C);

        if self.buffer.is_empty() {
            self.stop();
        } else {
            let index = self.fill_next_block();
            outb(self.bar1 + PO_LVI, index as u8);
        }
    }
}
