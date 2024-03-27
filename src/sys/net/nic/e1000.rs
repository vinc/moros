use crate::usr;
use crate::sys;
use crate::sys::allocator::PhysBuf;
use crate::sys::net::{EthernetDeviceIO, Config, Stats};
use spin::Mutex;

use alloc::sync::Arc;
use alloc::vec::Vec;
use bit_field::BitField;
use core::ptr;
use core::sync::atomic::{fence, AtomicUsize, Ordering};
use smoltcp::wire::EthernetAddress;
use x86_64::instructions::port::Port;
use x86_64::{PhysAddr, VirtAddr};

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

const REG_TIDV: u16 =   0x3820; // Transmit Interrupt Delay Value
const REG_TADV: u16 =   0x0382; // Transmit Absolute Interrupt Delay Value
const REG_RDTR: u16 =   0x2820; // Receive Delay Timer Register
const REG_RADV: u16 =   0x0282; // Receive Interrupt Absolute Delay Timer
const REG_ITR: u16 =    0x00C4; // Interrupt Throttling Register

const CTRL_LRST: u32 = 1 << 3;  // Link Reset
const CTRL_ASDE: u32 = 1 << 5;  // Auto-Speed Detection Enable
const CTRL_SLU: u32 =  1 << 6;  // Set Link Up
const CTRL_RST: u32 =  1 << 26; // Reset

const ICR_LSC: u32 =    1 << 2; // Link Status Change
const ICR_RXDMT0: u32 = 1 << 4; // Receive Descriptor Minimum Threshold Reached
const ICR_RXT0: u32 =   1 << 7; // Receiver Timer Interrupt

const RCTL_EN: u32 =            1 << 1;  // Receiver Enable
const RCTL_SBP: u32 =           1 << 2;  // Store Bad Packets
const RCTL_UPE: u32 =           1 << 3;  // Unicast Promiscuous Enabled
const RCTL_MPE: u32 =           1 << 4;  // Multicast Promiscuous Enabled
const RCTL_LPE: u32 =           1 << 5;  // Long Packet Reception Enable
const RCTL_LBM_NONE: u32 =      0 << 6;  // No Loopback
const RCTL_LBM_PHY: u32 =       3 << 6;  // PHY or external SerDesc loopback
const RTCL_RDMTS_HALF: u32 =    0 << 8;  // Free Buffer Threshold is 1/2 of RDLEN
const RTCL_RDMTS_QUARTER: u32 = 1 << 8;  // Free Buffer Threshold is 1/4 of RDLEN
const RTCL_RDMTS_EIGHTH: u32 =  2 << 8;  // Free Buffer Threshold is 1/8 of RDLEN
const RCTL_MO_36: u32 =         0 << 12; // Multicast Offset - bits 47:36
const RCTL_MO_35: u32 =         1 << 12; // Multicast Offset - bits 46:35
const RCTL_MO_34: u32 =         2 << 12; // Multicast Offset - bits 45:34
const RCTL_MO_32: u32 =         3 << 12; // Multicast Offset - bits 43:32
const RCTL_BAM: u32 =           1 << 15; // Broadcast Accept Mode
const RCTL_VFE: u32 =           1 << 18; // VLAN Filter Enable
const RCTL_CFIEN: u32 =         1 << 19; // Canonical Form Indicator Enable
const RCTL_CFI: u32 =           1 << 20; // Canonical Form Indicator Bit Value
const RCTL_DPF: u32 =           1 << 22; // Discard Pause Frames
const RCTL_PMCF: u32 =          1 << 23; // Pass MAC Control Frames
const RCTL_SECRC: u32 =         1 << 26; // Strip Ethernet CRC

// Buffer Sizes
const RCTL_BSIZE_256: u32 =     3 << 16;
const RCTL_BSIZE_512: u32 =     2 << 16;
const RCTL_BSIZE_1024: u32 =    1 << 16;
const RCTL_BSIZE_2048: u32 =    0 << 16;
const RCTL_BSIZE_4096: u32 =    (3 << 16) | (1 << 25);
const RCTL_BSIZE_8192: u32 =    (2 << 16) | (1 << 25);
const RCTL_BSIZE_16384: u32 =   (1 << 16) | (1 << 25);

const CMD_EOP: u8 =            1 << 0;  // End of Packet
const CMD_IFCS: u8 =           1 << 1;  // Insert FCS
const CMD_IC: u8 =             1 << 2;  // Insert Checksum
const CMD_RS: u8 =             1 << 3;  // Report Status
const CMD_RPS: u8 =            1 << 4;  // Report Packet Sent
const CMD_VLE: u8 =            1 << 6;  // VLAN Packet Enable
const CMD_IDE: u8 =            1 << 7;  // Interrupt Delay Enable

const TCTL_EN: u32 =            1 << 1;  // Transmit Enable
const TCTL_PSP: u32 =           1 << 3;  // Pad Short Packets
const TCTL_CT_SHIFT: u32 =           4;  // Collision Threshold
const TCTL_COLD_SHIFT: u32 =        12;  // Collision Distance
const TCTL_SWXOFF: u32 =        1 << 22; // Software XOFF Transmission
const TCTL_RTLC: u32 =          1 << 24; // Re-transmit on Late Collision

const TSTA_DD: u32 =             1 << 0;  // Descriptor Done
const TSTA_EC: u32 =             1 << 1;  // Excess Collisions
const TSTA_LC: u32 =             1 << 2;  // Late Collision
const LSTA_TU: u32 =             1 << 3;  // Transmit Underrun

const IO_ADDR: u16 = 0x00;
const IO_DATA: u16 = 0x04;

const MTU: usize = 1500;

// NOTE: Must be a multiple of 8
const RX_BUFFERS_COUNT: usize = 8;
const TX_BUFFERS_COUNT: usize = 8;

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
        let mut device = Self {
            bar_type: bar_type,
            io_base: io_base,
            mem_base: mem_base,
            has_eeprom: false,
            config: Arc::new(Config::new()),
            stats: Arc::new(Stats::new()),
            rx_buffers: [(); RX_BUFFERS_COUNT].map(|_| PhysBuf::new(2048)),
            tx_buffers: [(); TX_BUFFERS_COUNT].map(|_| PhysBuf::new(2048)),
            rx_descs: Arc::new(Mutex::new([(); RX_BUFFERS_COUNT].map(|_| RxDesc::default()))),
            tx_descs: Arc::new(Mutex::new([(); TX_BUFFERS_COUNT].map(|_| TxDesc::default()))),
            //rx_descs: [RxDesc::default(); RX_BUFFERS_COUNT],
            //tx_descs: [TxDesc::default(); TX_BUFFERS_COUNT],
            rx_id: Arc::new(AtomicUsize::new(0)),

            // Before a transmission begin the id is incremented,
            // so the first transimission will start at 0.
            tx_id: Arc::new(AtomicUsize::new(TX_BUFFERS_COUNT - 1)),
        };
        device.init();
        device
    }

    fn init(&mut self) {
        //debug!("NET E1000: CTRL:   {:#034b}", self.read(REG_CTRL));
        //debug!("NET E1000: IMS:    {:#034b}", self.read(REG_IMS));
        //debug!("NET E1000: ICR:    {:#034b}", self.read(REG_ICR));
        //debug!("NET E1000: STATUS: {:#034b}", self.read(REG_STATUS));

        //self.write(REG_IMS, 0); // Disable interrupts
        self.write(REG_IMC, 0xFFFFFFFF); // Disable interrupts

        let ctrl = self.read(REG_CTRL);
        self.write(REG_CTRL, ctrl | CTRL_RST); // Reset
        //debug!("NET E1000: CTRL:   {:#034b}", self.read(REG_CTRL));
        sys::time::nanowait(500);
        //debug!("NET E1000: CTRL:   {:#034b}", self.read(REG_CTRL));
        let ctrl = self.read(REG_CTRL) & !CTRL_LRST;
        self.write(REG_CTRL, ctrl); // Link Reset

        //self.write(REG_IMS, 0); // Disable interrupts
        self.write(REG_IMC, 0xFFFFFFFF); // Disable interrupts
        self.read(REG_ICR); // Clear interrupts

        //self.config.update_mac(EthernetAddress::from_bytes(&[0, 0, 0, 0, 0, 0]));
        self.detect_eeprom();

        //debug!("NET E1000: io base: {}", self.io_base);
        //debug!("NET E1000: mem base: {:#X}", self.mem_base.as_u64());
        //debug!("NET E1000: bar type: {:#X}", self.bar_type);
        if self.has_eeprom {
            //debug!("NET E1000: eeprom available");
        } else {
            //debug!("NET E1000: eeprom unavailable");
        }

        //debug!("NET E1000: CTRL:   {:#034b}", self.read(REG_CTRL));
        //debug!("NET E1000: IMS:    {:#034b}", self.read(REG_IMS));
        //debug!("NET E1000: ICR:    {:#034b}", self.read(REG_ICR));
        //debug!("NET E1000: STATUS: {:#034b}", self.read(REG_STATUS));

        self.config.update_mac(self.read_mac());

        //self.write(REG_IMS, 0x1F6DC);
        //self.write(REG_IMS, 0xff & !4);

        self.init_rx();
        self.init_tx();
        fence(Ordering::SeqCst);
        self.link_up();

        //self.write(REG_TIDV, 0);
        //self.write(REG_TADV, 0);
        //self.write(REG_RDTR, 0);
        //self.write(REG_RADV, 0);
        //self.write(REG_ITR, 0);
        //self.write(REG_IMS, 1 << 7);
        //self.write(REG_ICR, 0);

        //debug!("NET E1000: STATUS: {:#034b}", self.read(REG_STATUS));
    }

    fn init_rx(&mut self) {
        let mut rx_descs = self.rx_descs.lock();
        // Multicast Table Array
        for i in 0..128 {
            self.write(REG_MTA + i * 4, 0);
        }

        let mut phys_addr_begin = 0;
        let mut phys_addr_end = 0;
        let n = RX_BUFFERS_COUNT;
        for i in 0..n {
            rx_descs[i].addr = self.rx_buffers[i].addr();
            rx_descs[i].status = 0;

            let ptr = ptr::addr_of!(rx_descs[i]) as *const u8;
            assert_eq!(((ptr as usize) / 16) * 16, ptr as usize);
            let p1 = sys::allocator::phys_addr(ptr);
            assert_eq!(((p1 as usize) / 16) * 16, p1 as usize);
            let p2 = sys::allocator::phys_addr(((ptr as usize) + 16) as *const u8);
            assert_eq!(p2 - p1, 16);

            if i == 0 {
                phys_addr_begin = p1;
            } else if i == n - 1 {
                phys_addr_end = p2;
            }

            //debug!("NET E1000: {:?} ({}: {:#X}..{:#X})", rx_descs[i], i, p1, p2);
        }
        //debug!("NET E1000: RxDesc: {:#X}..{:#X}", phys_addr_begin, phys_addr_end);
        assert_eq!(phys_addr_end - phys_addr_begin, (n as u64) * 16);

        assert_eq!((rx_descs.len() * 16) % 128, 0);

        let ptr = ptr::addr_of!(rx_descs[0]) as *const u8;
        let phys_addr = sys::allocator::phys_addr(ptr);

        //self.write(REG_RDBAL, phys_addr.get_bits(0..32) as u32);
        //self.write(REG_RDBAH, phys_addr.get_bits(32..64) as u32);
        self.write(REG_RDBAL, phys_addr as u32);
        self.write(REG_RDBAH, (phys_addr >> 32) as u32);

        self.write(REG_RDLEN, (n as u32) * 16);

        self.write(REG_RDH, 0);
        self.write(REG_RDT, n as u32);
        //self.write(REG_RDT, (n as u32) - 1);

        //self.write(REG_RCTL, RCTL_EN | RCTL_SBP | RCTL_UPE | RCTL_MPE | RCTL_LBM_NONE | RTCL_RDMTS_HALF | RCTL_BAM | RCTL_SECRC | RCTL_BSIZE_8192);
        self.write(REG_RCTL, RCTL_EN | RCTL_BAM | RCTL_SECRC | RCTL_BSIZE_2048);

        //debug!("NET E1000: STATUS: {:#034b}", self.read(REG_STATUS));
        fence(Ordering::SeqCst);
    }

    fn init_tx(&mut self) {
        let mut tx_descs = self.tx_descs.lock();
        let mut phys_addr_begin = 0;
        let mut phys_addr_end = 0;
        let n = TX_BUFFERS_COUNT;
        for i in 0..n {
            tx_descs[i].addr = self.tx_buffers[i].addr();
            tx_descs[i].cmd = 0;
            tx_descs[i].status = TSTA_DD as u8;

            let ptr = ptr::addr_of!(tx_descs[i]) as *const _;
            assert_eq!(((ptr as usize) / 16) * 16, ptr as usize);
            let p1 = sys::allocator::phys_addr(ptr);
            assert_eq!(((p1 as usize) / 16) * 16, p1 as usize);
            let p2 = sys::allocator::phys_addr(((ptr as usize) + 16) as *const _);
            assert_eq!(p2 - p1, 16);

            if i == 0 {
                phys_addr_begin = p1;
            } else if i == n - 1 {
                phys_addr_end = p2;
            }

            //debug!("NET E1000: {:?} ({}: {:#X}..{:#X})", tx_descs[i], i, p1, p2);
        }
        //debug!("NET E1000: TxDesc: {:#X}..{:#X}", phys_addr_begin, phys_addr_end);
        assert_eq!(phys_addr_end - phys_addr_begin, (n as u64) * 16);

        assert_eq!((tx_descs.len() * 16) % 128, 0);

        let ptr = ptr::addr_of!(tx_descs[0]) as *const _;
        let phys_addr = sys::allocator::phys_addr(ptr);

        //self.write(REG_TDBAL, phys_addr.get_bits(0..32) as u32);
        //self.write(REG_TDBAH, phys_addr.get_bits(32..64) as u32);
        self.write(REG_TDBAL, phys_addr as u32);
        self.write(REG_TDBAH, (phys_addr >> 32) as u32);

        self.write(REG_TDLEN, (n as u32) * 16);

        self.write(REG_TDH, 0);
        self.write(REG_TDT, 0);

        //self.write(REG_TCTL, TCTL_EN | TCTL_PSP | (0x10 << TCTL_CT_SHIFT) | (0x40 << TCTL_COLD_SHIFT) | TCTL_RTLC);
        self.write(REG_TCTL, TCTL_EN | TCTL_PSP | (0x10 << TCTL_CT_SHIFT) | (0x40 << TCTL_COLD_SHIFT));
        self.write(REG_TIPG, 10 | (8 << 10) | (6 << 20));

        //self.write(REG_TCTL, 0b0110000000000111111000011111010);
        //self.write(REG_TIPG, 0x0060200A);

        //debug!("NET E1000: STATUS: {:#034b}", self.read(REG_STATUS));
        fence(Ordering::SeqCst);
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
                let addr = sys::mem::phys_to_virt(self.mem_base + 0x5400 as u64).as_u64();
                let mac_32 = core::ptr::read_volatile(addr as *const u32);
                if mac_32 != 0 {
                    let mac_8 = alloc::slice::from_raw_parts(addr as *const u8, 6);
                    mac[..].clone_from_slice(mac_8);
                }
            }
        }
        EthernetAddress::from_bytes(&mac[..])
    }

    fn link_up(&self) {
        let ctrl = self.read(REG_CTRL);
        self.write(REG_CTRL, ctrl | CTRL_SLU | CTRL_ASDE & !CTRL_LRST);
        //debug!("NET E1000: STATUS: {:#034b}", self.read(REG_STATUS));
    }

    fn write(&self, addr: u16, data: u32) {
        unsafe {
            if self.bar_type == 0 {
                let addr = sys::mem::phys_to_virt(self.mem_base + addr as u64).as_u64() as *mut u32;
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
                let addr = sys::mem::phys_to_virt(self.mem_base + addr as u64).as_u64() as *const u32;
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
}

impl EthernetDeviceIO for Device {
    fn config(&self) -> Arc<Config> {
        self.config.clone()
    }

    fn stats(&self) -> Arc<Stats> {
        self.stats.clone()
    }

    fn receive_packet(&mut self) -> Option<Vec<u8>> {
        //debug!("------------------------------------------------------------");
        //debug!("NET E1000: receive_packet");
        let tx_descs = self.tx_descs.lock();
        for i in 0..TX_BUFFERS_COUNT {
            let rx_descs = self.rx_descs.lock();
            let ptr = ptr::addr_of!(rx_descs[i]) as *const u8;
            assert_eq!(((ptr as usize) / 16) * 16, ptr as usize);
            let phy = sys::allocator::phys_addr(ptr);
            //debug!("NET E1000: [{}] {:?} ({:#X} -> {:#X})", i, tx_descs[i], ptr as u64, phy);
        }
        fence(Ordering::SeqCst);
        //debug!("NET E1000: CTRL:   {:#034b}", self.read(REG_CTRL));
        let icr = self.read(REG_ICR);
        self.write(REG_ICR, icr);
        //debug!("NET E1000: ICR:    {:#034b}", icr);
        //debug!("NET E1000: STATUS: {:#034b}", self.read(REG_STATUS));
        //debug!("NET E1000: RDH: {}", self.read(REG_RDH));
        //debug!("NET E1000: RDT: {}", self.read(REG_RDT));

        //self.write(REG_IMS, 0x1);
        if icr & ICR_LSC > 0 {
            //debug!("NET E1000: ICR.LSC");
            self.link_up();
        } else if icr & ICR_RXDMT0 > 0 {
            //debug!("NET E1000: ICR.RXDMT0");
        } else if icr & ICR_RXT0 > 0 {
            //debug!("NET E1000: ICR.RXT0");

            let rx_id = self.rx_id.load(Ordering::SeqCst);
            //debug!("NET E1000: rx_id = {}", rx_id);

            let rx_descs = self.rx_descs.lock();
            //debug!("NET E1000: {:?}", rx_descs[rx_id]);

            fence(Ordering::SeqCst);
            self.rx_id.store((rx_id + 1) % RX_BUFFERS_COUNT, Ordering::SeqCst);

            let n = rx_descs[rx_id].len as usize;
            return Some(self.rx_buffers[rx_id][0..n].to_vec());
        }

        /*
        for i in 0..RX_BUFFERS_COUNT {
            fence(Ordering::SeqCst);
            let rx_descs = self.rx_descs.lock();
            debug!("NET E1000: [{}] {:?}", i, rx_descs[i]);
            let mut n = 0;
            for (j, b) in self.rx_buffers[i].iter().enumerate() {
                if *b != 0 {
                    n = j;
                }
            }
            if n > 0 {
                debug!("NET E1000: RX_BUFFER[{}]", i);
                usr::hex::print_hex(&self.rx_buffers[i][0..n]);
            }
        }
        */

        None
    }

    fn transmit_packet(&mut self, len: usize) {
        //debug!("------------------------------------------------------------");
        //debug!("NET E1000: transmit_packet({})", len);
        let tx_id = self.tx_id.load(Ordering::SeqCst);
        //debug!("NET E1000: tx_id = {}", tx_id);
        //debug!("NET E1000: TDH: {}", self.read(REG_TDH));
        //debug!("NET E1000: TDT: {}", self.read(REG_TDT));
        //debug!("NET E1000: {:?}", tx_descs[tx_id]);

        //usr::hex::print_hex(&self.tx_buffers[tx_id][0..len]);

        fence(Ordering::SeqCst);
        let mut tx_descs = self.tx_descs.lock();
        assert_eq!(tx_descs[tx_id].addr, self.tx_buffers[tx_id].addr());
        tx_descs[tx_id].len = len as u16;
        tx_descs[tx_id].cmd = CMD_EOP | CMD_IFCS | CMD_RS;
        tx_descs[tx_id].status = 0;

        //debug!("NET E1000: {:?}", tx_descs[tx_id]);

        /*
        for i in 0..TX_BUFFERS_COUNT {
            let ptr = ptr::addr_of!(tx_descs[i]) as *const u8;
            assert_eq!(((ptr as usize) / 16) * 16, ptr as usize);
            let phy = sys::allocator::phys_addr(ptr);
            debug!("NET E1000: [{}] {:?} ({:#X} -> {:#X})", i, tx_descs[i], ptr as u64, phy);
            //debug!("NET E1000: [{}] {:?}", i, tx_descs[i]);
        }
        */

        fence(Ordering::SeqCst);
        self.write(REG_TDT, ((tx_id + 1) % TX_BUFFERS_COUNT) as u32);
        //debug!("NET E1000: TDT <- {}", tx_id + 1);

        /*
        for i in 0..256 {
            debug!("NET E1000: [{}] {:?} ({})", tx_id, tx_descs[tx_id], i);
            sys::time::nanowait(50000);
            self.read(REG_STATUS);
            fence(Ordering::SeqCst);
            if tx_descs[tx_id].status == 1 {
                break;
            }
        }
        */

        //debug!("NET E1000: STATUS: {:#034b}", self.read(REG_STATUS));
        //fence(Ordering::SeqCst);
    }

    fn next_tx_buffer(&mut self, len: usize) -> &mut [u8] {
        //debug!("------------------------------------------------------------");
        //debug!("NET E1000: next_tx_buffer");
        let tx_id = (self.tx_id.load(Ordering::SeqCst) + 1) % TX_BUFFERS_COUNT;
        //debug!("NET E1000: tx_id = {}", tx_id);
        self.tx_id.store(tx_id, Ordering::SeqCst);
        //self.write(REG_TDT, (tx_id + 1) as u32); // FIXME?
        &mut self.tx_buffers[tx_id][0..len]
    }
}

#[test_case]
fn test_driver() {
    assert_eq!(core::mem::size_of::<RxDesc>(), 16);
    assert_eq!(core::mem::size_of::<TxDesc>(), 16);
}
