use crate::sys;
use crate::sys::allocator::PhysBuf;
use crate::sys::net::{EthernetDeviceIO, Config, Stats};
use spin::Mutex;

use alloc::slice;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bit_field::BitField;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};
use smoltcp::wire::EthernetAddress;
use x86_64::instructions::port::Port;
use x86_64::PhysAddr;

// https://pdos.csail.mit.edu/6.828/2019/readings/hardware/8254x_GBe_SDM.pdf

// Registers
const REG_CTRL: u16 =   0x0000; // Device Control Register
const REG_STATUS: u16 = 0x0008; // Device Status Register
const REG_EECD: u16 =   0x0014; // EEPROM/Flash Control & Data Register
const REG_ICR: u16 =    0x00C0; // Interrupt Cause Read Register
const REG_IMS: u16 =    0x00D0; // Interrupt Mask Set/Read Register
const REG_IMC: u16 =    0x00D8; // Interrupt Mask Clear Register
const REG_RCTL: u16 =   0x0100; // Receive Control Register
const REG_RDBAL: u16 =  0x2800; // Receive Descriptor Base Address Low
const REG_RDBAH: u16 =  0x2804; // Receive Descriptor Base Address High
const REG_RDLEN: u16 =  0x2808; // Receive Descriptor Length
const REG_RDH: u16 =    0x2810; // Receive Descriptor Head
const REG_RDT: u16 =    0x2818; // Receive Descriptor Tail
const REG_TCTL: u16 =   0x0400; // Transmit Control Register
const REG_TIPG: u16 =   0x0410; // Transmit IPG Register
const REG_TDBAL: u16 =  0x3800; // Transmit Descriptor Base Address Low
const REG_TDBAH: u16 =  0x3804; // Transmit Descriptor Base Address High
const REG_TDLEN: u16 =  0x3808; // Transmit Descriptor Length
const REG_TDH: u16 =    0x3810; // Transmit Descriptor Head
const REG_TDT: u16 =    0x3818; // Transmit Descriptor Tail
const REG_MTA: u16 =    0x5200; // Multicast Table Array

const CTRL_LRST: u32 = 1 << 3;  // Link Reset
const CTRL_ASDE: u32 = 1 << 5;  // Auto-Speed Detection Enable
const CTRL_SLU: u32 =  1 << 6;  // Set Link Up
const CTRL_RST: u32 =  1 << 26; // Reset

const ICR_LSC: u32 =    1 << 2; // Link Status Change
const ICR_RXDMT0: u32 = 1 << 4; // Receive Descriptor Minimum Threshold Reached
const ICR_RXT0: u32 =   1 << 7; // Receiver Timer Interrupt

const RCTL_EN: u32 =    1 << 1;  // Receiver Enable
const RCTL_BAM: u32 =   1 << 15; // Broadcast Accept Mode
const RCTL_SECRC: u32 = 1 << 26; // Strip Ethernet CRC

// Buffer Sizes
// const RCTL_BSIZE_256: u32 =    3 << 16;
// const RCTL_BSIZE_512: u32 =    2 << 16;
// const RCTL_BSIZE_1024: u32 =   1 << 16;
// const RCTL_BSIZE_2048: u32 =   0 << 16;
// const RCTL_BSIZE_4096: u32 =  (3 << 16) | (1 << 25);
// const RCTL_BSIZE_16384: u32 = (1 << 16) | (1 << 25);
const RCTL_BSIZE_8192: u32 = (2 << 16) | (1 << 25);

const CMD_EOP: u8 =  1 << 0; // End of Packet
const CMD_IFCS: u8 = 1 << 1; // Insert FCS
const CMD_RS: u8 =   1 << 3; // Report Status

const TCTL_EN: u32 = 1 << 1;     // Transmit Enable
const TCTL_PSP: u32 = 1 << 3;    // Pad Short Packets
const TCTL_MULR: u32 = 1 << 28;  // Multiple Request Support
const TCTL_CT_SHIFT: u32 = 4;    // Collision Threshold
const TCTL_COLD_SHIFT: u32 = 12; // Collision Distance

// Transmit Descriptor Status Field
const TSTA_DD: u8 = 1 << 0; // Descriptor Done

// Receive Descriptor Status Field
const RSTA_DD: u8 =  1 << 0; // Descriptor Done
const RSTA_EOP: u8 = 1 << 1; // End of Packet

// Device Status Register
const DSTA_LU: u32 = 1 << 1; // Link Up Indication

// Transmit IPG Register
const TIPG_IPGT: u32 = 10; // IPG Transmit Time
const TIPG_IPGR1: u32 = 8; // IPG Receive Time 1
const TIPG_IPGR2: u32 = 6; // IPG Receive Time 2

const IO_ADDR: u16 = 0x00;
const IO_DATA: u16 = 0x04;

// NOTE: Must be a multiple of 8
const RX_BUFFERS_COUNT: usize = 64;
const TX_BUFFERS_COUNT: usize = 8;

// NOTE: Must be equals
const BUFFER_SIZE: usize = 8192;
const RCTL_BSIZE: u32 = RCTL_BSIZE_8192;

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, align(16))]
struct RxDesc {
    addr: u64,
    len: u16,
    checksum: u16,
    status: u8,
    errors: u8,
    special: u16,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, align(16))]
struct TxDesc {
    addr: u64,
    len: u16,
    cso: u8,
    cmd: u8,
    status: u8,
    css: u8,
    special: u16,
}

#[derive(Clone)]
pub struct Device {
    mem_base: PhysAddr,
    io_base: u16,
    bar_type: u16,
    has_eeprom: bool,
    config: Arc<Config>,
    stats: Arc<Stats>,
    rx_buffers: [PhysBuf; RX_BUFFERS_COUNT],
    tx_buffers: [PhysBuf; TX_BUFFERS_COUNT],
    rx_descs: Arc<Mutex<[RxDesc; RX_BUFFERS_COUNT]>>,
    tx_descs: Arc<Mutex<[TxDesc; TX_BUFFERS_COUNT]>>,
    rx_id: Arc<AtomicUsize>,
    tx_id: Arc<AtomicUsize>,
}

impl Device {
    pub fn new(io_base: u16, mem_base: PhysAddr, bar_type: u16) -> Self {
        const RX: usize = RX_BUFFERS_COUNT;
        const TX: usize = TX_BUFFERS_COUNT;

        let mut device = Self {
            bar_type: bar_type,
            io_base: io_base,
            mem_base: mem_base,
            has_eeprom: false,
            config: Arc::new(Config::new()),
            stats: Arc::new(Stats::new()),
            rx_buffers: [(); RX].map(|_| PhysBuf::new(BUFFER_SIZE)),
            tx_buffers: [(); TX].map(|_| PhysBuf::new(BUFFER_SIZE)),
            rx_descs: Arc::new(Mutex::new([(); RX].map(|_| RxDesc::default()))),
            tx_descs: Arc::new(Mutex::new([(); TX].map(|_| TxDesc::default()))),
            rx_id: Arc::new(AtomicUsize::new(0)),

            // Before a transmission begin the id is incremented,
            // so the first transimission will start at 0.
            tx_id: Arc::new(AtomicUsize::new(TX - 1)),
        };
        device.reset();
        device.init();
        device
    }

    fn reset(&mut self) {
        // Disable interrupts
        self.write(REG_IMC, 0xFFFF);

        // Reset device
        let ctrl = self.read(REG_CTRL);
        self.write(REG_CTRL, ctrl | CTRL_RST); // Reset
        sys::time::nanowait(500); // TODO: How long should we wait?

        // Disable interrupts again
        self.write(REG_IMC, 0xFFFF);

        // Reset link
        let ctrl = self.read(REG_CTRL) & !CTRL_LRST;
        self.write(REG_CTRL, ctrl);
    }

    fn init(&mut self) {
        self.detect_eeprom();
        self.config.update_mac(self.read_mac());

        self.init_rx();
        self.init_tx();
        self.link_up();

        // TODO: Enable interrupts
        //self.write(REG_IMS, ICR_LSC | ICR_RXDMT0 | ICR_RXT0);
        self.write(REG_IMS, 0);

        // Clear interrupts
        self.read(REG_ICR);
    }

    fn init_rx(&mut self) {
        // Multicast Table Array
        for i in 0..128 {
            self.write(REG_MTA + i * 4, 0);
        }

        // Descriptors
        let mut rx_descs = self.rx_descs.lock();
        let n = RX_BUFFERS_COUNT;
        for i in 0..n {
            rx_descs[i].addr = self.rx_buffers[i].addr();
            rx_descs[i].status = 0;
        }

        let ptr = ptr::addr_of!(rx_descs[0]) as *const u8;
        let phys_addr = sys::allocator::phys_addr(ptr);

        // Ring address and length
        self.write(REG_RDBAL, phys_addr.get_bits(0..32) as u32);
        self.write(REG_RDBAH, phys_addr.get_bits(32..64) as u32);
        self.write(REG_RDLEN, (n * 16) as u32);

        // Ring head and tail
        self.write(REG_RDH, 0);
        self.write(REG_RDT, (n - 1) as u32);

        // Control Register
        self.write(REG_RCTL, RCTL_EN | RCTL_BAM | RCTL_SECRC | RCTL_BSIZE);
    }

    fn init_tx(&mut self) {
        // Descriptors
        let mut tx_descs = self.tx_descs.lock();
        let n = TX_BUFFERS_COUNT;
        for i in 0..n {
            tx_descs[i].addr = self.tx_buffers[i].addr();
            tx_descs[i].cmd = 0;
            tx_descs[i].status = TSTA_DD as u8;
        }

        let ptr = ptr::addr_of!(tx_descs[0]) as *const _;
        let phys_addr = sys::allocator::phys_addr(ptr);

        // Ring address and length
        self.write(REG_TDBAL, phys_addr.get_bits(0..32) as u32);
        self.write(REG_TDBAH, phys_addr.get_bits(32..64) as u32);
        self.write(REG_TDLEN, (n as u32) * 16);

        // Ring head and tail
        self.write(REG_TDH, 0);
        self.write(REG_TDT, 0);

        // Control Register
        // NOTE: MULR is only needed for Intel I217-LM
        self.write(REG_TCTL, TCTL_EN    // Transmit Enable
            | TCTL_PSP                  // Pad Short Packets
            | (0x0F << TCTL_CT_SHIFT)   // Collision Threshold
            | (0x3F << TCTL_COLD_SHIFT) // Collision Distance
            | TCTL_MULR);               // Multiple Request Support

        // Inter Packet Gap (3 x 10 bits)
        self.write(REG_TIPG, TIPG_IPGT  // IPG Transmit Time
            | (TIPG_IPGR1 << 10)        // IPG Receive Time 1
            | (TIPG_IPGR2 << 20));      // IPG Receive Time 2
    }

    fn read_mac(&self) -> EthernetAddress {
        let mut mac = [0; 6];
        if self.has_eeprom {
            let mut tmp;
            tmp = self.read_eeprom(0);
            mac[0] = (tmp &0xff) as u8;
            mac[1] = (tmp >> 8) as u8;
            tmp = self.read_eeprom(1);
            mac[2] = (tmp &0xff) as u8;
            mac[3] = (tmp >> 8) as u8;
            tmp = self.read_eeprom(2);
            mac[4] = (tmp &0xff) as u8;
            mac[5] = (tmp >> 8) as u8;
        } else {
            unsafe {
                let phys = self.mem_base + 0x5400;
                let addr = sys::mem::phys_to_virt(phys).as_u64();
                let mac_32 = core::ptr::read_volatile(addr as *const u32);
                if mac_32 != 0 {
                    let mac_8 = slice::from_raw_parts(addr as *const u8, 6);
                    mac[..].clone_from_slice(mac_8);
                }
            }
        }
        EthernetAddress::from_bytes(&mac[..])
    }

    fn link_up(&self) {
        let ctrl = self.read(REG_CTRL);
        self.write(REG_CTRL, ctrl | CTRL_SLU | CTRL_ASDE & !CTRL_LRST);
    }

    fn write(&self, addr: u16, data: u32) {
        unsafe {
            if self.bar_type == 0 {
                let phys = self.mem_base + addr as u64;
                let addr = sys::mem::phys_to_virt(phys).as_u64() as *mut u32;
                core::ptr::write_volatile(addr, data);
            } else {
                Port::new(self.io_base + IO_ADDR).write(addr);
                Port::new(self.io_base + IO_DATA).write(data);
            }
        }
    }

    fn read(&self, addr: u16) -> u32 {
        unsafe {
            if self.bar_type == 0 {
                let phys = self.mem_base + addr as u64;
                let addr = sys::mem::phys_to_virt(phys).as_u64() as *mut u32;
                core::ptr::read_volatile(addr)
            } else {
                Port::new(self.io_base + IO_ADDR).write(addr);
                Port::new(self.io_base + IO_DATA).read()
            }
        }
    }

    fn detect_eeprom(&mut self) {
        self.write(REG_EECD, 1);
        let mut i = 0;
        while !self.has_eeprom && i < 1000 {
            self.has_eeprom = self.read(REG_EECD) & 0x10 > 0;
            i += 1;
        }
    }

    fn read_eeprom(&self, addr: u16) -> u32 {
        let e = if self.has_eeprom { 4 } else { 0 };
        self.write(REG_EECD, 1 | ((addr as u32) << 2 * e));
        let mut res = 0;
        while res & (1 << 1 * e) == 0 {
            res = self.read(REG_EECD);
        }
        (res >> 16) & 0xFFFF
    }

    #[allow(dead_code)]
    fn debug(&self) {
        // Registers
        debug!("NET E1000: ICR:    {:#034b}", self.read(REG_ICR));
        debug!("NET E1000: CTRL:   {:#034b}", self.read(REG_CTRL));
        debug!("NET E1000: STATUS: {:#034b}", self.read(REG_STATUS));
        debug!("NET E1000: RDH -> {}", self.read(REG_RDH));
        debug!("NET E1000: RDT -> {}", self.read(REG_RDT));
        debug!("NET E1000: TDH -> {}", self.read(REG_TDH));
        debug!("NET E1000: TDT -> {}", self.read(REG_TDT));

        // Receive descriptors
        let rx_descs = self.rx_descs.lock();
        for i in 0..RX_BUFFERS_COUNT {
            let ptr = ptr::addr_of!(rx_descs[i]) as *const u8;
            let phy = sys::allocator::phys_addr(ptr);
            debug!(
                "NET E1000: [{}] {:?} ({:#X} -> {:#X})",
                i, rx_descs[i], ptr as u64, phy
            );
        }

        // Transmit descriptors
        let tx_descs = self.tx_descs.lock();
        for i in 0..TX_BUFFERS_COUNT {
            let ptr = ptr::addr_of!(tx_descs[i]) as *const u8;
            let phy = sys::allocator::phys_addr(ptr);
            debug!(
                "NET E1000: [{}] {:?} ({:#X} -> {:#X})",
                i, tx_descs[i], ptr as u64, phy
            );
        }
    }
}

impl EthernetDeviceIO for Device {
    fn config(&self) -> Arc<Config> {
        self.config.clone()
    }

    fn stats(&self) -> Arc<Stats> {
        self.stats.clone()
    }

    fn receive_packet(&mut self) -> Option<Vec<u8>> {
        let icr = self.read(REG_ICR);
        self.write(REG_ICR, icr);

        // Link Status Change
        if icr & ICR_LSC > 0 {
            if self.read(REG_STATUS) & DSTA_LU == 0 {
                self.link_up();
                return None;
            }
        }

        // Receive Descriptor Minimum Threshold
        if icr & ICR_RXDMT0 > 0 {
            // TODO
        }

        // Receiver Timer Interrupt
        if icr & ICR_RXT0 > 0 {
            // TODO
        }

        let rx_id = self.rx_id.load(Ordering::SeqCst);
        let mut rx_descs = self.rx_descs.lock();

        // If hardware is done with the current descriptor
        if rx_descs[rx_id].status & RSTA_DD > 0 {
            if rx_descs[rx_id].status & RSTA_EOP == 0 {
                // FIXME: this is not the last descriptor for the packet
            }
            self.rx_id.store((rx_id + 1) % RX_BUFFERS_COUNT, Ordering::SeqCst);
            let n = rx_descs[rx_id].len as usize;
            let buf = self.rx_buffers[rx_id][0..n].to_vec();
            rx_descs[rx_id].status = 0; // Driver is done
            self.write(REG_RDT, rx_id as u32);
            return Some(buf);
        }

        None
    }

    fn transmit_packet(&mut self, len: usize) {
        let tx_id = self.tx_id.load(Ordering::SeqCst);
        let mut tx_descs = self.tx_descs.lock();
        debug_assert_eq!(tx_descs[tx_id].addr, self.tx_buffers[tx_id].addr());

        // Setup descriptor
        tx_descs[tx_id].len = len as u16;
        tx_descs[tx_id].cmd = CMD_EOP | CMD_IFCS | CMD_RS;
        tx_descs[tx_id].status = 0; // Driver is done

        // Let the hardware handle the descriptor
        self.write(REG_TDT, ((tx_id + 1) % TX_BUFFERS_COUNT) as u32);
    }

    fn next_tx_buffer(&mut self, len: usize) -> &mut [u8] {
        let tx_id = (self.tx_id.load(Ordering::SeqCst) + 1) % TX_BUFFERS_COUNT;
        self.tx_id.store(tx_id, Ordering::SeqCst);
        &mut self.tx_buffers[tx_id][0..len]
    }
}

#[test_case]
fn test_driver() {
    assert_eq!(core::mem::size_of::<RxDesc>(), 16);
    assert_eq!(core::mem::size_of::<TxDesc>(), 16);
}
