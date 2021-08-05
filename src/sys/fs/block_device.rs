use super::block_bitmap::BlockBitmap;
use super::dir::Dir;

use crate::sys;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

pub const MAGIC: &str = "MOROS FS";

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
    disk: Vec<[u8; super::BLOCK_SIZE]>,
}

impl MemBlockDevice {
    pub fn new(len: usize) -> Self {
        let disk = vec![[0; super::BLOCK_SIZE]; len];
        Self { disk }
    }
}

impl BlockDeviceIO for MemBlockDevice {
    fn read(&self, block_index: u32, buf: &mut [u8]) {
        buf[..].clone_from_slice(&self.disk[block_index as usize][..]);
    }

    fn write(&mut self, block_index: u32, buf: &[u8]) {
        self.disk[block_index as usize][..].clone_from_slice(&buf[..]);
    }
}

pub struct AtaBlockDevice {
    bus: u8,
    dsk: u8,
}

impl AtaBlockDevice {
    pub fn new(bus: u8, dsk: u8) -> Self {
        Self { bus, dsk }
    }
}

impl BlockDeviceIO for AtaBlockDevice {
    fn read(&self, block_addr: u32, mut buf: &mut [u8]) {
        sys::ata::read(self.bus, self.dsk, block_addr, &mut buf);
    }

    fn write(&mut self, block_addr: u32, buf: &[u8]) {
        sys::ata::write(self.bus, self.dsk, block_addr, buf);
    }
}

pub fn is_mounted() -> bool {
    BLOCK_DEVICE.lock().is_some()
}

pub fn mount_ata(bus: u8, dsk: u8) {
    let dev = AtaBlockDevice::new(bus, dsk);
    *BLOCK_DEVICE.lock() = Some(BlockDevice::Ata(dev));
}

// NOTE: format_ata will also call mount_ata to bootstrap the root dir
pub fn format_ata(bus: u8, dsk: u8) {
    // Write superblock
    let mut buf = MAGIC.as_bytes().to_vec();
    buf.resize(super::BLOCK_SIZE, 0);
    let mut dev = AtaBlockDevice::new(bus, dsk);
    dev.write(super::SUPERBLOCK_ADDR, &buf);

    mount_ata(bus, dsk);

    // Allocate root dir
    let root = Dir::root();
    BlockBitmap::alloc(root.addr());
}

pub fn init() {
    for bus in 0..2 {
        for dsk in 0..2 {
            let mut buf = [0u8; super::BLOCK_SIZE];
            sys::ata::read(bus, dsk, super::SUPERBLOCK_ADDR, &mut buf);
            if String::from_utf8_lossy(&buf[0..8]) == MAGIC {
                log!("MFS Superblock found in ATA {}:{}\n", bus, dsk);
                mount_ata(bus, dsk);
                return;
            }
        }
    }
}
