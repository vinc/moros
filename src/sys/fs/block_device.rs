use super::bitmap_block::BitmapBlock;
use super::dir::Dir;
use super::super_block::SuperBlock;

use crate::sys;

use alloc::vec;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref BLOCK_DEVICE: Mutex<Option<BlockDevice>> = Mutex::new(None);
}

pub enum BlockDevice {
    Mem(MemBlockDevice),
    Ata(AtaBlockDevice),
}

pub trait BlockDeviceIO {
    fn read(&self, addr: u32, buf: &mut [u8]) -> Result<(), ()>;
    fn write(&mut self, addr: u32, buf: &[u8]) -> Result<(), ()>;
    fn block_size(&self) -> usize;
    fn block_count(&self) -> usize;
}

impl BlockDeviceIO for BlockDevice {
    fn read(&self, addr: u32, buf: &mut [u8]) -> Result<(), ()> {
        match self {
            BlockDevice::Mem(dev) => dev.read(addr, buf),
            BlockDevice::Ata(dev) => dev.read(addr, buf),
        }
    }

    fn write(&mut self, addr: u32, buf: &[u8]) -> Result<(), ()> {
        match self {
            BlockDevice::Mem(dev) => dev.write(addr, buf),
            BlockDevice::Ata(dev) => dev.write(addr, buf),
        }
    }

    fn block_size(&self) -> usize {
        match self {
            BlockDevice::Mem(dev) => dev.block_size() as usize,
            BlockDevice::Ata(dev) => dev.block_size() as usize,
        }
    }

    fn block_count(&self) -> usize {
        match self {
            BlockDevice::Mem(dev) => dev.block_count() as usize,
            BlockDevice::Ata(dev) => dev.block_count() as usize,
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

    /*
    pub fn len(&self) -> usize {
        self.dev.len()
    }
    */
}

impl BlockDeviceIO for MemBlockDevice {
    fn read(&self, block_index: u32, buf: &mut [u8]) -> Result<(), ()> {
        // TODO: check for overflow
        buf[..].clone_from_slice(&self.dev[block_index as usize][..]);
        Ok(())
    }

    fn write(&mut self, block_index: u32, buf: &[u8]) -> Result<(), ()> {
        // TODO: check for overflow
        self.dev[block_index as usize][..].clone_from_slice(&buf[..]);
        Ok(())
    }

    fn block_size(&self) -> usize {
        super::BLOCK_SIZE
    }

    fn block_count(&self) -> usize {
        self.dev.len()
    }
}

pub fn mount_mem() {
    let mem = sys::allocator::memory_size() / 2; // Half the allocatable memory
    let len = mem / super::BLOCK_SIZE; // TODO: take a size argument
    let dev = MemBlockDevice::new(len);
    *BLOCK_DEVICE.lock() = Some(BlockDevice::Mem(dev));
}

pub fn format_mem() {
    debug_assert!(is_mounted());
    if let Some(sb) = SuperBlock::new() {
        sb.write();
        let root = Dir::root();
        BitmapBlock::alloc(root.addr());
    }
}

#[derive(Clone)]
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

    /*
    pub fn len(&self) -> usize {
        self.block_size() * self.block_count()
    }
    */
}

impl BlockDeviceIO for AtaBlockDevice {
    fn read(&self, block_addr: u32, mut buf: &mut [u8]) -> Result<(), ()> {
        sys::ata::read(self.dev.bus, self.dev.dsk, block_addr, &mut buf)
    }

    fn write(&mut self, block_addr: u32, buf: &[u8]) -> Result<(), ()> {
        sys::ata::write(self.dev.bus, self.dev.dsk, block_addr, buf)
    }

    fn block_size(&self) -> usize {
        self.dev.block_size() as usize
    }

    fn block_count(&self) -> usize {
        self.dev.block_count() as usize
    }
}

pub fn mount_ata(bus: u8, dsk: u8) {
    *BLOCK_DEVICE.lock() = AtaBlockDevice::new(bus, dsk).map(|dev| BlockDevice::Ata(dev));
}

pub fn format_ata() {
    if let Some(sb) = SuperBlock::new() {
        // Write super_block
        sb.write();

        // Write zeros into block bitmaps
        super::bitmap_block::free_all();

        // Allocate root dir
        debug_assert!(is_mounted());
        let root = Dir::root();
        BitmapBlock::alloc(root.addr());
    }
}

pub fn is_mounted() -> bool {
    BLOCK_DEVICE.lock().is_some()
}

pub fn dismount() {
    *BLOCK_DEVICE.lock() = None;
}

#[test_case]
fn test_mount_mem() {
    assert!(!is_mounted());
    mount_mem();
    assert!(is_mounted());
    dismount();
}
