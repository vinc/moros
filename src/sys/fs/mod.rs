mod block;
mod bitmap_block;
mod block_device;
mod device;
mod dir;
mod dir_entry;
mod file;
mod read_dir;
mod super_block;

pub use device::{Device, DeviceType};
pub use dir::Dir;
pub use dir_entry::FileStat;
pub use file::{File, SeekFrom};
pub use block_device::{format_ata, format_mem, is_mounted, mount_ata, mount_mem, dismount};
pub use crate::api::fs::{dirname, filename, realpath, FileIO};
pub use crate::sys::ata::BLOCK_SIZE;

use dir_entry::DirEntry;
use super_block::SuperBlock;

pub const VERSION: u8 = 1;

#[repr(u8)]
pub enum OpenFlag {
    Read   = 1,
    Write  = 2,
    Create = 4,
    Dir    = 8,
    Device = 16,
}

impl OpenFlag {
    fn is_set(self, flags: usize) -> bool {
        flags & (self as usize) != 0
    }
}

pub fn open(path: &str, flags: usize) -> Option<Resource> {
    if OpenFlag::Dir.is_set(flags) {
        let res = Dir::open(path);
        if res.is_none() && OpenFlag::Create.is_set(flags) {
            Dir::create(path)
        } else {
            res
        }.map(|r| Resource::Dir(r))
    } else if OpenFlag::Device.is_set(flags) {
        let res = Device::open(path);
        if res.is_none() && OpenFlag::Create.is_set(flags) {
            Device::create(path)
        } else {
            res
        }.map(|r| Resource::Device(r))
    } else {
        let res = File::open(path);
        if res.is_none() && OpenFlag::Create.is_set(flags) {
            File::create(path)
        } else {
            res
        }.map(|r| Resource::File(r))
    }
}

pub fn stat(pathname: &str) -> Option<FileStat> {
    DirEntry::open(pathname).map(|e| e.stat())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Dir = 0,
    File = 1,
    Device = 2,
}

#[derive(Debug, Clone)]
pub enum Resource {
    Dir(Dir),
    File(File),
    Device(Device),
}

impl FileIO for Resource {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        match self {
            Resource::Dir(io) => io.read(buf),
            Resource::File(io) => io.read(buf),
            Resource::Device(io) => io.read(buf),
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        match self {
            Resource::Dir(io) => io.write(buf),
            Resource::File(io) => io.write(buf),
            Resource::Device(io) => io.write(buf),
        }
    }
}

// TODO: All this should be done dynamically
// We could store the disk size in the super_block area
// And we could maybe also have a counter of allocated block in there to make
// disk usage report O(1)
const DISK_SIZE: usize = (8 << 20) / BLOCK_SIZE; // 8 MB disk
const KERNEL_SIZE: usize = (2 << 20) / BLOCK_SIZE; // 2 MB for the kernel binary
const MAX_BLOCKS: usize = (DISK_SIZE - KERNEL_SIZE) / 2; // FIXME: Replace `/ 2` with `- SUPELBLOCK_AREA_SIZE - BITMAP_AREA_SIZE`
const SUPERBLOCK_ADDR: u32 = KERNEL_SIZE as u32; // Address of the block
const BITMAP_ADDR: u32 = SUPERBLOCK_ADDR + 2;
const DATA_ADDR: u32 = BITMAP_ADDR + ((MAX_BLOCKS as u32) / bitmap_block::BITMAP_SIZE as u32 / 8); // 1 bit per block in bitmap

pub fn disk_size() -> usize {
    (SuperBlock::read().block_count as usize) * BLOCK_SIZE
}

pub fn disk_used() -> usize {
    (SuperBlock::read().alloc_count as usize) * BLOCK_SIZE
}

pub fn disk_free() -> usize {
    disk_size() - disk_used()
}

pub fn init() {
    /*
    printk!("disk size       = {} blocks\n", DISK_SIZE);
    printk!("kernel size     = {} blocks\n", KERNEL_SIZE);
    printk!("super_block addr = {}\n", SUPERBLOCK_ADDR);
    printk!("bitmap addr     = {}\n", BITMAP_ADDR);
    printk!("data addr       = {}\n", DATA_ADDR);
    printk!("end addr        = {}\n", DATA_ADDR + MAX_BLOCKS as u32);
    */

    block_device::init();
}
