use super::block_bitmap::BlockBitmap;
use super::dir::Dir;
use super::super_block::SuperBlock;

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
    fn block_size(&self) -> usize;
    fn block_count(&self) -> usize;
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
    fn read(&self, block_index: u32, buf: &mut [u8]) {
        buf[..].clone_from_slice(&self.dev[block_index as usize][..]);
    }

    fn write(&mut self, block_index: u32, buf: &[u8]) {
        self.dev[block_index as usize][..].clone_from_slice(&buf[..]);
    }

    fn block_size(&self) -> usize {
        super::BLOCK_SIZE
    }

    fn block_count(&self) -> usize {
        self.dev.len()
    }
}

pub fn mount_mem() {
    let len = super::DISK_SIZE / 2;
    let dev = MemBlockDevice::new(len);
    *BLOCK_DEVICE.lock() = Some(BlockDevice::Mem(dev));
}

pub fn format_mem() {
    debug_assert!(is_mounted());
    if let Some(sb) = SuperBlock::new() {
        sb.write();
        let root = Dir::root();
        BlockBitmap::alloc(root.addr());
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
    fn read(&self, block_addr: u32, mut buf: &mut [u8]) {
        sys::ata::read(self.dev.bus, self.dev.dsk, block_addr, &mut buf);
    }

    fn write(&mut self, block_addr: u32, buf: &[u8]) {
        sys::ata::write(self.dev.bus, self.dev.dsk, block_addr, buf);
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
        super::block_bitmap::free_all();

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
                log!("MFS SuperBlock found in ATA {}:{}\n", bus, dsk);
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
