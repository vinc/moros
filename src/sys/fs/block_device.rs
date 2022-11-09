use super::bitmap_block::BitmapBlock;
use super::dir::Dir;
use super::super_block::SuperBlock;

use crate::sys;

use alloc::vec;
use alloc::vec::Vec;
use spin::Mutex;

pub static BLOCK_DEVICE: Mutex<Option<BlockDevice>> = Mutex::new(None);

pub enum BlockDevice {
    Mem(MemBlockDevice),
    Ata(AtaBlockDevice),
}

pub trait BlockDeviceIO {
    fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), ()>;
    fn write(&mut self, addr: u32, buf: &[u8]) -> Result<(), ()>;
    fn block_size(&self) -> usize;
    fn block_count(&self) -> usize;
}

impl BlockDeviceIO for BlockDevice {
    fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), ()> {
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
            BlockDevice::Mem(dev) => dev.block_size(),
            BlockDevice::Ata(dev) => dev.block_size(),
        }
    }

    fn block_count(&self) -> usize {
        match self {
            BlockDevice::Mem(dev) => dev.block_count(),
            BlockDevice::Ata(dev) => dev.block_count(),
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
    fn read(&mut self, block_index: u32, buf: &mut [u8]) -> Result<(), ()> {
        // TODO: check for overflow
        buf[..].clone_from_slice(&self.dev[block_index as usize][..]);
        Ok(())
    }

    fn write(&mut self, block_index: u32, buf: &[u8]) -> Result<(), ()> {
        // TODO: check for overflow
        self.dev[block_index as usize][..].clone_from_slice(buf);
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

const ATA_CACHE_SIZE: usize = 1024;

#[derive(Clone)]
pub struct AtaBlockDevice {
    cache: [Option<(u32, Vec<u8>)>; ATA_CACHE_SIZE],
    dev: sys::ata::Drive
}

impl AtaBlockDevice {
    pub fn new(bus: u8, dsk: u8) -> Option<Self> {
        sys::ata::Drive::open(bus, dsk).map(|dev| {
            let cache = [(); ATA_CACHE_SIZE].map(|_| None);
            Self { dev, cache }
        })
    }

    /*
    pub fn len(&self) -> usize {
        self.block_size() * self.block_count()
    }
    */

    fn hash(&self, block_addr: u32) -> usize {
        (block_addr as usize) % self.cache.len()
    }

    fn cached_block(&self, block_addr: u32) -> Option<&[u8]> {
        let h = self.hash(block_addr);
        if let Some((cached_addr, cached_buf)) = &self.cache[h] {
            if block_addr == *cached_addr {
                return Some(cached_buf);
            }
        }
        None
    }

    fn set_cached_block(&mut self, block_addr: u32, buf: &[u8]) {
        let h = self.hash(block_addr);
        self.cache[h] = Some((block_addr, buf.to_vec()));
    }

    fn unset_cached_block(&mut self, block_addr: u32) {
        let h = self.hash(block_addr);
        self.cache[h] = None;
    }
}

impl BlockDeviceIO for AtaBlockDevice {
    fn read(&mut self, block_addr: u32, buf: &mut [u8]) -> Result<(), ()> {
        if let Some(cached) = self.cached_block(block_addr) {
            buf.copy_from_slice(cached);
            return Ok(());
        }

        sys::ata::read(self.dev.bus, self.dev.dsk, block_addr, buf)?;
        self.set_cached_block(block_addr, buf);
        Ok(())
    }

    fn write(&mut self, block_addr: u32, buf: &[u8]) -> Result<(), ()> {
        sys::ata::write(self.dev.bus, self.dev.dsk, block_addr, buf)?;
        self.unset_cached_block(block_addr);
        Ok(())
    }

    fn block_size(&self) -> usize {
        self.dev.block_size() as usize
    }

    fn block_count(&self) -> usize {
        self.dev.block_count() as usize
    }
}

pub fn mount_ata(bus: u8, dsk: u8) {
    *BLOCK_DEVICE.lock() = AtaBlockDevice::new(bus, dsk).map(BlockDevice::Ata);
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
