use super::block_bitmap::BlockBitmap;
use super::dir::Dir;
use super::superblock_addr;

use crate::sys;

use lazy_static::lazy_static;
use spin::Mutex;

pub const MAGIC: &str = "MOROS FS";

lazy_static! {
    pub static ref BLOCK_DEVICE: Mutex<Option<BlockDevice>> = Mutex::new(None);
}

pub struct BlockDevice {
    bus: u8,
    dsk: u8,
}

impl BlockDevice {
    pub fn new(bus: u8, dsk: u8) -> Self {
        Self { bus, dsk }
    }

    pub fn read(&self, block: u32, mut buf: &mut [u8]) {
        sys::ata::read(self.bus, self.dsk, block, &mut buf);
    }

    pub fn write(&self, block: u32, buf: &[u8]) {
        sys::ata::write(self.bus, self.dsk, block, buf);
    }
}

pub fn is_mounted() -> bool {
    BLOCK_DEVICE.lock().is_some()
}

pub fn mount(bus: u8, dsk: u8) {
    let block_device = BlockDevice::new(bus, dsk);
    *BLOCK_DEVICE.lock() = Some(block_device);
}

pub fn format(bus: u8, dsk: u8) {
    // Write superblock
    let mut buf = MAGIC.as_bytes().to_vec();
    buf.resize(512, 0);
    let block_device = BlockDevice::new(bus, dsk);
    block_device.write(superblock_addr(), &buf);

    mount(bus, dsk);

    // Allocate root dir
    let root = Dir::root();
    BlockBitmap::alloc(root.addr());
}
