use crate::{sys, usr};
use crate::sys::allocator::PhysBuf;
use crate::sys::net::Stats;
use crate::sys::net::EthernetDeviceIO;

use alloc::sync::Arc;
use alloc::vec::Vec;
use bit_field::BitField;
use core::sync::atomic::{AtomicUsize, Ordering};
use smoltcp::wire::EthernetAddress;
use x86_64::instructions::port::Port;

const CSR0_INIT: usize = 0;
const CSR0_STRT: usize = 1;
//const CSR0_STOP: usize = 2;
const CSR0_TDMD: usize = 3;
//const CSR0_TXON: usize = 4;
//const CSR0_RXON: usize = 5;
//const CSR0_IENA: usize = 6;
//const CSR0_INTR: usize = 7;
const CSR0_IDON: usize = 8;
//const CSR0_TINT: usize = 9;
//const CSR0_RINT: usize = 10;
//const CSR0_MERR: usize = 11;
//const CSR0_MISS: usize = 12;
//const CSR0_CERR: usize = 13;
//const CSR0_BABL: usize = 14;
//const CSR0_ERR: usize = 0;

const DE_ENP:  usize = 0;
const DE_STP:  usize = 1;
//const DE_BUFF: usize = 2;
//const DE_CRC:  usize = 3;
//const DE_OFLO: usize = 4;
//const DE_FRAM: usize = 5;
//const DE_ERR:  usize = 6;
const DE_OWN:  usize = 7;

#[derive(Clone)]
pub struct Ports {
    pub mac: [Port<u8>; 6],
    pub rdp_16: Port<u16>,
    pub rap_16: Port<u16>,
    pub rst_16: Port<u16>,
    pub bdp_16: Port<u16>,

    pub rdp_32: Port<u32>,
    pub rap_32: Port<u32>,
    pub rst_32: Port<u32>,
    pub bdp_32: Port<u32>,
}

impl Ports {
    pub fn new(io_base: u16) -> Self {
        Self {
            mac: [
                Port::new(io_base + 0x00),
                Port::new(io_base + 0x01),
                Port::new(io_base + 0x02),
                Port::new(io_base + 0x03),
                Port::new(io_base + 0x04),
                Port::new(io_base + 0x05),
            ],

            rdp_16: Port::new(io_base + 0x10),
            rap_16: Port::new(io_base + 0x12),
            rst_16: Port::new(io_base + 0x14),
            bdp_16: Port::new(io_base + 0x16),

            rdp_32: Port::new(io_base + 0x10),
            rap_32: Port::new(io_base + 0x14),
            rst_32: Port::new(io_base + 0x18),
            bdp_32: Port::new(io_base + 0x1C),
        }
    }

    fn write_rap_32(&mut self, val: u32) {
        unsafe { self.rap_32.write(val) }
    }

    fn read_csr_32(&mut self, csr: u32) -> u32 {
        self.write_rap_32(csr);
        unsafe { self.rdp_32.read() }
    }

    fn write_csr_32(&mut self, csr: u32, val: u32) {
        self.write_rap_32(csr);
        unsafe { self.rdp_32.write(val) }
    }

    fn read_bcr_32(&mut self, bcr: u32) -> u32 {
        self.write_rap_32(bcr);
        unsafe { self.bdp_32.read() }
    }

    fn write_bcr_32(&mut self, bcr: u32, val: u32) {
        self.write_rap_32(bcr);
        unsafe { self.bdp_32.write(val) }
    }

    fn mac(&mut self) -> [u8; 6] {
        unsafe {
            [
                self.mac[0].read(),
                self.mac[1].read(),
                self.mac[2].read(),
                self.mac[3].read(),
                self.mac[4].read(),
                self.mac[5].read(),
            ]
        }
    }
}

const MTU: usize = 1536;
const RX_BUFFERS_COUNT: usize = 32;
const TX_BUFFERS_COUNT: usize = 8;
const DE_LEN: usize = 16;

fn log2(x: u8) -> u8 {
    8 - 1 - x.leading_zeros() as u8
}

fn is_buffer_owner(des: &PhysBuf, i: usize) -> bool {
    // This bit indicates that the descriptor entry is owned
    // by the host (OWN=0) or by the PCnet-PCI controller (OWN=1)
    !des[DE_LEN * i + 7].get_bit(DE_OWN)
}

#[derive(Clone)]
pub struct Device {
    pub debug_mode: bool,
    pub stats: Stats,
    ports: Ports,
    eth_addr: Option<EthernetAddress>,

    rx_buffers: [PhysBuf; RX_BUFFERS_COUNT],
    tx_buffers: [PhysBuf; TX_BUFFERS_COUNT],
    rx_des: PhysBuf, // Ring buffer of rx descriptor entries
    tx_des: PhysBuf, // Ring buffer of tx descriptor entries
    rx_id: Arc<AtomicUsize>,
    tx_id: Arc<AtomicUsize>,
}

impl Device {
    pub fn new(io_base: u16) -> Self {
        Self {
            debug_mode: false,
            stats: Stats::new(),
            ports: Ports::new(io_base),
            eth_addr: None,
            rx_buffers: [(); RX_BUFFERS_COUNT].map(|_| PhysBuf::new(MTU)),
            tx_buffers: [(); TX_BUFFERS_COUNT].map(|_| PhysBuf::new(MTU)),
            rx_des: PhysBuf::new(RX_BUFFERS_COUNT * DE_LEN),
            tx_des: PhysBuf::new(TX_BUFFERS_COUNT * DE_LEN),
            rx_id: Arc::new(AtomicUsize::new(0)),
            tx_id: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn init_descriptor_entry(&mut self, i: usize, is_rx: bool) {
        let des = if is_rx { &mut self.rx_des } else { &mut self.tx_des };

        // Set buffer address
        let addr = if is_rx {
            self.rx_buffers[i].addr().to_le_bytes()
        } else {
            self.tx_buffers[i].addr().to_le_bytes()
        };
        des[DE_LEN * i + 0] = addr[0];
        des[DE_LEN * i + 1] = addr[1];
        des[DE_LEN * i + 2] = addr[2];
        des[DE_LEN * i + 3] = addr[3];

        // Set buffer byte count (0..12 BCNT + 12..16 ONES)
        let bcnt = (0xF000 | (0x0FFF & (1 + !(MTU as u16)))).to_le_bytes();
        des[DE_LEN * i + 4] = bcnt[0];
        des[DE_LEN * i + 5] = bcnt[1];

        if is_rx {
            des[DE_LEN * i + 7].set_bit(DE_OWN, true); // Set ownership to card
        }
    }
}

impl EthernetDeviceIO for Device {
    fn init(&mut self) {
        // Read MAC addr
        let mac = self.ports.mac();
        self.eth_addr = Some(EthernetAddress::from_bytes(&mac));

        // Reset to 16-bit access
        unsafe {
            self.ports.rst_32.read();
            self.ports.rst_16.read();
        }

        // Switch to 32-bit access
        unsafe {
            self.ports.rdp_32.write(0);
        }

        // SWSTYLE
        let mut csr_58 = self.ports.read_csr_32(58);
        csr_58 &= 0xFF00;
        csr_58 |= 2;
        self.ports.write_csr_32(58, csr_58);

        // ASEL
        let mut bcr_2 = self.ports.read_bcr_32(2);
        bcr_2 |= 2;
        self.ports.write_bcr_32(2, bcr_2);

        // Initialize ring buffers
        let is_rx = true;
        for i in 0..RX_BUFFERS_COUNT {
            self.init_descriptor_entry(i, is_rx);
        }
        for i in 0..TX_BUFFERS_COUNT {
            self.init_descriptor_entry(i, !is_rx);
        }

        // Card register setup
        let mut init_struct = PhysBuf::new(28);
        init_struct[0] = 0; // Mode
        init_struct[1] = 0; // Mode
        init_struct[2].set_bits(4..8, log2(RX_BUFFERS_COUNT as u8));
        init_struct[3].set_bits(4..8, log2(TX_BUFFERS_COUNT as u8));
        init_struct[4] = mac[0];
        init_struct[5] = mac[1];
        init_struct[6] = mac[2];
        init_struct[7] = mac[3];
        init_struct[8] = mac[4];
        init_struct[9] = mac[5];
        let rx_addr = self.rx_des.addr().to_le_bytes();
        init_struct[20] = rx_addr[0];
        init_struct[21] = rx_addr[1];
        init_struct[22] = rx_addr[2];
        init_struct[23] = rx_addr[3];
        let tx_addr = self.tx_des.addr().to_le_bytes();
        init_struct[24] = tx_addr[0];
        init_struct[25] = tx_addr[1];
        init_struct[26] = tx_addr[2];
        init_struct[27] = tx_addr[3];
        let addr = init_struct.addr();
        self.ports.write_csr_32(1, addr.get_bits(0..16) as u32);
        self.ports.write_csr_32(2, addr.get_bits(16..32) as u32);
        assert!(self.ports.read_csr_32(0) == 0b000000100); // STOP

        // self.ports.write_csr_32(4, 1 << 11); // Pad short ethernet packets

        // Set INIT bit
        self.ports.write_csr_32(0, 1 << CSR0_INIT);

        // Wait until init is done
        while !self.ports.read_csr_32(0).get_bit(CSR0_IDON) {
            sys::time::halt();
        }
        assert!(self.ports.read_csr_32(0) == 0b110000001); // IDON + INTR + INIT

        // Start the card
        self.ports.write_csr_32(0, 1 << CSR0_STRT);
        assert!(self.ports.read_csr_32(0) == 0b110110011); // IDON + INTR + RXON + TXON + STRT + INIT
    }

    fn mac(&self) -> Option<EthernetAddress> {
        self.eth_addr
    }

    fn receive_packet(&mut self) -> Option<Vec<u8>> {
        let mut packet = Vec::new();
        let mut rx_id = self.rx_id.load(Ordering::SeqCst);
        while is_buffer_owner(&self.rx_des, rx_id) {
            if self.debug_mode {
                printk!("{}\n", "-".repeat(66));
                log!("NET PCNET Receiving:\n");
                //printk!("CSR0: {:016b}\n", self.ports.read_csr_32(0));
                //printk!("RX Buffer: {}\n", rx_id);
            }

            let rmd1 = self.rx_des[rx_id * DE_LEN + 7];
            let end_of_packet = rmd1.get_bit(DE_ENP);

            /*
            let start_of_packet = rmd1.get_bit(DE_STP);
            let error = rmd1.get_bit(DE_ERR);
            let buffer_error = rmd1.get_bit(DE_BUFF);
            let overflow_error = rmd1.get_bit(DE_OFLO) && !rmd1.get_bit(DE_ENP);
            let crc_error = rmd1.get_bit(DE_CRC) && !rmd1.get_bit(DE_OFLO) && rmd1.get_bit(DE_ENP);
            let framing_error = rmd1.get_bit(DE_FRAM) && !rmd1.get_bit(DE_OFLO) && rmd1.get_bit(DE_ENP);

            if self.debug_mode {
                printk!("Flags: ");
                if start_of_packet {
                    printk!("start_of_packet ");
                }
                if end_of_packet {
                    printk!("end_of_packet ");
                }
                if error {
                    if overflow_error {
                        printk!("overflow_error ");
                    }
                    if framing_error {
                        printk!("framing_error ");
                    }
                    if crc_error {
                        printk!("crc_error ");
                    }
                }
                printk!("\n");
            }
            */

            // Read packet size
            let packet_size = u16::from_le_bytes([
                self.rx_des[rx_id * DE_LEN + 8],
                self.rx_des[rx_id * DE_LEN + 9]
            ]) as usize;

            let n = if end_of_packet { packet_size } else { self.rx_buffers[rx_id].len() };
            packet.extend(&self.rx_buffers[rx_id][0..n]);

            self.rx_des[rx_id * DE_LEN + 7].set_bit(DE_OWN, true); // Give back ownership
            rx_id = (rx_id + 1) % RX_BUFFERS_COUNT;
            self.rx_id.store(rx_id, Ordering::Relaxed);

            if end_of_packet {
                break;
            }
        }

        if !packet.is_empty() {
            self.stats.rx_add(packet.len() as u64);
            if self.debug_mode {
                //printk!("Size: {} bytes\n", packet.len());
                usr::hex::print_hex(&packet);
                //printk!("CSR0: {:016b}\n", self.ports.read_csr_32(0));
                //printk!("RDTE: {:016b}\n", self.rx_des[rx_id * DE_LEN + 7]);
            }
            Some(packet)
        } else {
            None
        }
    }

    fn transmit_packet(&mut self, len: usize) {
        let tx_id = self.tx_id.load(Ordering::SeqCst);

        self.tx_des[tx_id * DE_LEN + 7].set_bit(DE_STP, true); // Set start of packet
        self.tx_des[tx_id * DE_LEN + 7].set_bit(DE_ENP, true); // Set end of packet
        // Set buffer byte count (0..12 BCNT + 12..16 ONES)
        let bcnt = (0xF000 | (0x0FFF & (1 + !(len as u16)))).to_le_bytes();
        self.tx_des[tx_id * DE_LEN + 4] = bcnt[0];
        self.tx_des[tx_id * DE_LEN + 5] = bcnt[1];
        // Give back ownership to the card
        self.tx_des[tx_id * DE_LEN + 7].set_bit(DE_OWN, true);
        self.tx_id.store((tx_id + 1) % TX_BUFFERS_COUNT, Ordering::Relaxed);

        if is_buffer_owner(&self.tx_des, tx_id) {
            if self.debug_mode {
                printk!("{}\n", "-".repeat(66));
                log!("NET PCNET Transmitting:\n");
                //printk!("TX Buffer: {}\n", tx_id);
                //printk!("CSR0: {:016b}\n", self.ports.read_csr_32(0));
            }
        } else {
            self.ports.write_csr_32(0, 1 << CSR0_TDMD); // Send all buffers
        }
    }

    fn next_tx_buffer(&mut self, len: usize) -> &mut [u8] {
        let tx_id = self.tx_id.load(Ordering::SeqCst);
        &mut self.tx_buffers[tx_id][0..len]
    }
}
