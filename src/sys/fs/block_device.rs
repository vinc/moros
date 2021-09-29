use super::block_bitmap::BlockBitmap;
use super::dir::Dir;

use crate::sys;

use alloc::vec;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

pub const SIGNATURE: &[u8; 8] = b"MOROS FS";

lazy_static! {
    pub static ref BLOCK_DEVICE: Mutex<Option<BlockDevice>> = Mutex::new(None);
}

pub enum BlockDevice {
    Mem(MemBlockDevice),
    Ata(AtaBlockDevice),
}

pub trait BlockDeviceIO {
    fn read(&self, addr: u32, buf: &mut [u8]);
    fn write(&mut self, addr: u32, buf: &[u8]);
}

impl BlockDeviceIO for BlockDevice {
    fn read(&self, addr: u32, buf: &mut [u8]) {
        match self {
            BlockDevice::Mem(dev) => dev.read(addr, buf),
            BlockDevice::Ata(dev) => dev.read(addr, buf),
        }
    }

    fn write(&mut self, addr: u32, buf: &[u8]) {
        match self {
            BlockDevice::Mem(dev) => dev.write(addr, buf),
            BlockDevice::Ata(dev) => dev.write(addr, buf),
        }
    }
}

pub struct MemBlockDevice {
    dev: Vec<[u8; super::BLOCK_SIZE]>,
}

impl MemBlockDevice {
    pub fn new(len: usize) -> Self {
        let dev = vec![[0; super::BLOCK_SIZE]; len];
        Self { dev }
    }
}

impl BlockDeviceIO for MemBlockDevice {
    fn read(&self, block_index: u32, buf: &mut [u8]) {
        buf[..].clone_from_slice(&self.dev[block_index as usize][..]);
    }

    fn write(&mut self, block_index: u32, buf: &[u8]) {
        self.dev[block_index as usize][..].clone_from_slice(&buf[..]);
    }
}

pub fn mount_mem() {
    let len = super::DISK_SIZE / 2;
    let dev = MemBlockDevice::new(len);
    *BLOCK_DEVICE.lock() = Some(BlockDevice::Mem(dev));
}

pub fn format_mem() {
    debug_assert!(is_mounted());
    let root = Dir::root();
    BlockBitmap::alloc(root.addr());
}

pub struct AtaBlockDevice {
    dev: sys::ata::Drive
}

impl AtaBlockDevice {
    pub fn new(bus: u8, dsk: u8) -> Option<Self> {
        if let Some(dev) = sys::ata::Drive::identify(bus, dsk) {
            Some(Self { dev })
        } else {
            None
        }
    }

    pub fn block_size(&self) -> usize {
        self.dev.block_size() as usize
    }

    pub fn block_count(&self) -> usize {
        self.dev.block_count() as usize
    }
}

impl BlockDeviceIO for AtaBlockDevice {
    fn read(&self, block_addr: u32, mut buf: &mut [u8]) {
        sys::ata::read(self.dev.bus, self.dev.dsk, block_addr, &mut buf);
    }

    fn write(&mut self, block_addr: u32, buf: &[u8]) {
        sys::ata::write(self.dev.bus, self.dev.dsk, block_addr, buf);
    }
}

pub fn mount_ata(bus: u8, dsk: u8) {
    *BLOCK_DEVICE.lock() = AtaBlockDevice::new(bus, dsk).map(|dev| BlockDevice::Ata(dev));
}

pub fn format_ata(bus: u8, dsk: u8) {
    if let Some(mut dev) = AtaBlockDevice::new(bus, dsk) {
        // Write superblock
        let mut buf = [0; super::BLOCK_SIZE];
        buf[0..8].clone_from_slice(SIGNATURE);
        let count = dev.block_count() as u32;
        let size = dev.block_size() as u32;
        debug_assert!(size >= 512);
        debug_assert!(size.is_power_of_two());
        buf[8..12].clone_from_slice(&count.to_be_bytes());
        buf[12] = (size.trailing_zeros() as u8) - 9; // 2 ^ (9 + n)
        dev.write(super::SUPERBLOCK_ADDR, &buf);

        // Write zeros into block bitmaps
        let buf = vec![0; super::BLOCK_SIZE];
        for addr in super::BITMAP_ADDR..super::DATA_ADDR {
            dev.write(addr, &buf);
        }

        // Allocate root dir
        debug_assert!(is_mounted());
        let root = Dir::root();
        BlockBitmap::alloc(root.addr());
    }
}

pub fn is_mounted() -> bool {
    BLOCK_DEVICE.lock().is_some()
}

pub fn dismount() {
    *BLOCK_DEVICE.lock() = None;
}

pub fn init() {
    for bus in 0..2 {
        for dsk in 0..2 {
            let mut buf = [0u8; super::BLOCK_SIZE];
            sys::ata::read(bus, dsk, super::SUPERBLOCK_ADDR, &mut buf);
            if &buf[0..8] == SIGNATURE {
                log!("MFS Superblock found in ATA {}:{}\n", bus, dsk);
                mount_ata(bus, dsk);
                return;
            }
        }
    }
}

#[test_case]
fn test_mount_mem() {
    assert!(!is_mounted());
    mount_mem();
    assert!(is_mounted());
    dismount();
}
