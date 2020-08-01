use crate::{kernel, log};
use crate::kernel::allocator::PhysBuf;
use crate::kernel::net::State;
use array_macro::array;
use bit_field::BitField;
use smoltcp::iface::{EthernetInterfaceBuilder, NeighborCache, Routes};
use smoltcp::phy;
use smoltcp::phy::{Device, DeviceCapabilities};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, IpCidr, Ipv4Address};
use smoltcp::Result;
use x86_64::instructions::port::Port;

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

const MTU: usize = 1520;
const RX_BUFFERS_COUNT: usize = 32;
const TX_BUFFERS_COUNT: usize = 8;
const DE_LEN: usize = 16;

fn log2(x: u8) -> u8 {
    x.leading_zeros() as u8
}

fn is_buffer_owner(des: &PhysBuf, i: usize) -> bool {
    (des[DE_LEN * i + 7] & 0x80) == 0
}

pub struct PCNET {
    pub debug_mode: bool,
    pub state: State,
    ports: Ports,
    eth_addr: Option<EthernetAddress>,

    rx_buffers: [PhysBuf; RX_BUFFERS_COUNT],
    tx_buffers: [PhysBuf; TX_BUFFERS_COUNT],
    rx_des: PhysBuf, // Ring buffer of rx descriptor entries
    tx_des: PhysBuf, // Ring buffer of tx descriptor entries
    rx_id: usize,
    tx_id: usize,
}

impl PCNET {
    pub fn new(io_base: u16) -> Self {
        Self {
            debug_mode: false,
            state: State::new(),
            ports: Ports::new(io_base),
            eth_addr: None,
            rx_buffers: array![PhysBuf::new(MTU); RX_BUFFERS_COUNT],
            tx_buffers: array![PhysBuf::new(MTU); TX_BUFFERS_COUNT],
            rx_des: PhysBuf::new(RX_BUFFERS_COUNT * DE_LEN),
            tx_des: PhysBuf::new(TX_BUFFERS_COUNT * DE_LEN),
            rx_id: 0,
            tx_id: 0,
        }
    }

    fn init_descriptor_entry(&mut self, i: usize, is_rx: bool) {
        let des = if is_rx { &mut self.rx_des } else { &mut self.tx_des };

        let addr = if is_rx {
            self.rx_buffers[i].addr().to_le_bytes()
        } else {
            self.tx_buffers[i].addr().to_le_bytes()
        };
        des[DE_LEN * i + 0] = addr[0];
        des[DE_LEN * i + 1] = addr[1];
        des[DE_LEN * i + 2] = addr[2];
        des[DE_LEN * i + 3] = addr[3];

        let bnct = ((MTU as u16).reverse_bits() & 0x0FFF | 0xF000).to_le_bytes();
        des[DE_LEN * i + 4] = bnct[0];
        des[DE_LEN * i + 5] = bnct[1];

        if is_rx {
            des[DE_LEN * i + 7] = 0x80; // Set ownership to card
        }
    }

    pub fn init(&mut self) {
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
        init_struct[2] = log2(RX_BUFFERS_COUNT as u8);
        init_struct[3] = log2(TX_BUFFERS_COUNT as u8);
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
    }
}

pub fn init() {
    if let Some(mut pci_device) = kernel::pci::find_device(0x1022, 0x2000) {
        pci_device.enable_bus_mastering();
        let io_base = (pci_device.base_addresses[0] as u16) & 0xFFF0;
        let mut pcnet_device = PCNET::new(io_base);
        pcnet_device.init();
        if let Some(eth_addr) = pcnet_device.eth_addr {
            log!("NET PCNET MAC {}\n", eth_addr);
        }
    }
}
