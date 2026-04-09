use super::SndConfig;

use crate::sys;
use crate::sys::mem::PhysBuf;

use alloc::vec::Vec;
use spin::Mutex;
use x86_64::instructions::port::Port;

// Sources:
// https://wiki.osdev.org/Sound_Blaster_16
// https://pdos.csail.mit.edu/6.828/2006/readings/hardware/SoundBlaster.pdf

const MIXER_ADDR: u16 = 0x224;
const MIXER_DATA: u16 = 0x225;
const DSP_RESET:  u16 = 0x226;
const DSP_READ:   u16 = 0x22A;
const DSP_WRITE:  u16 = 0x22C;
const DSP_ACK:    u16 = 0x22E;

const IRQ: u8 = 5;

pub const BUF_LEN: usize = 32 << 10;

pub static SND: Mutex<Option<(PhysBuf, Vec<Vec<u8>>)>> = Mutex::new(None);

fn irq(num: u8) -> u8 {
    match num {
        2 => 0x01,
        5 => 0x02,
        7 => 0x04,
        10 => 0x08,
        _ => panic!(),
    }
}

fn outb(addr: u16, value: u8) {
    let mut port: Port<u8> = Port::new(addr);
    unsafe {
        port.write(value);
    }
}

fn inb(addr: u16) -> u8 {
    let mut port: Port<u8> = Port::new(addr);
    unsafe {
        port.read()
    }
}

fn reset() {
    outb(DSP_RESET, 1);
    sys::clk::wait(3000); // 3 microseconds
    outb(DSP_RESET, 0);
    loop {
        let res = inb(DSP_READ);
        if res == 0xAA {
            break;
        }
        // TODO: break after about 100 microseconds
    }
}

fn version() -> u8 {
    outb(DSP_WRITE, 0xE1);
    inb(DSP_READ)
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

pub fn stop() {
    if let Some((ref mut buf, ref mut queue)) = *SND.lock() {
        outb(DSP_WRITE, 0xD0); // Pause DMA playback
        let chan = 1;
        outb(0x0A, 0x04 + chan); // Disable channel
        buf.fill(0x80);
        queue.clear();
    }
}

// 8-bit Auto-initialize Transfer
pub fn play(buf: &[u8], config: &SndConfig) {
    debug_assert!(config.channels == 1);
    debug_assert!(config.sample_bits == 8);
    debug_assert!(config.sample_rate <= u16::MAX as u32);
    if let Some((ref mut block, ref mut queue)) = *SND.lock() {
        queue.clear();
        for chunk in buf.chunks(block.len()) {
            queue.push(chunk.to_vec());
        }
        let buf = queue.remove(0);
        let len = core::cmp::min(block.len(), buf.len());
        block[0..len].copy_from_slice(&buf[0..len]);
        block[len..].fill(0x80);

        // Program the DMA controller
        dma(block.addr(), block.size() - 1);

        // Set the DSP transfer sampling rate
        let rate = (config.sample_rate as u16).to_be_bytes();
        outb(DSP_WRITE, 0x41); // Output
        outb(DSP_WRITE, rate[0]); // High byte
        outb(DSP_WRITE, rate[1]); // Low byte

        // Send an I/O command
        outb(DSP_WRITE, 0xC6); // 8-bit output

        // Send the transfer mode
        outb(DSP_WRITE, 0x00); // 8-bit mono unsigned PCM

        // Send the DSP block transfer size
        let bytes = (block.size() - 1).to_le_bytes();
        outb(DSP_WRITE, bytes[0]);
        outb(DSP_WRITE, bytes[1]);
    }
}

pub fn init() {
    if version() != 0xFF {
        reset();

        // Set IRQ
        sys::idt::set_irq_handler(IRQ, interrupt_handler);
        outb(MIXER_ADDR, 0x80);
        outb(MIXER_DATA, irq(IRQ));

        let buf = PhysBuf::new(BUF_LEN);
        let queue = Vec::new();
        *SND.lock() = Some((buf, queue));
        log!("SND DRV SB16");
    }
}

fn interrupt_handler() {
    if let Some((ref mut buf, ref mut queue)) = *SND.lock() {
        if queue.is_empty() {
            outb(DSP_WRITE, 0xD0); // Pause

            let chan = 1;
            outb(0x0A, 0x04 + chan); // Disable channel
        } else {
            let pcm = queue.remove(0);
            let len = core::cmp::min(buf.len(), pcm.len());
            buf[0..len].copy_from_slice(&pcm[0..len]);
            buf[len..].fill(0x80);
        }
    }
    let _ = inb(DSP_ACK);
}
