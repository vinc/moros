mod block;
mod block_bitmap;
mod block_device;
mod dir;
mod dir_entry;
mod file;
mod read_dir;

pub use dir::Dir;
pub use file::{File, FileStat, SeekFrom};
pub use block_device::{format_ata, format_mem, is_mounted, mount_ata, mount_mem, dismount};
pub use crate::api::fs::{dirname, filename, realpath};

use block_bitmap::BlockBitmap;

#[repr(u8)]
pub enum OpenFlag {
    Read = 1,
    Write = 2,
    Create = 4,
}

pub fn open_file(path: &str, flags: usize) -> Option<File> {
    let res = File::open(path);
    if res.is_none() && flags & (OpenFlag::Create as usize) != 0 {
        File::create(path)
    } else {
        res
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Dir = 0,
    File = 1,
}

// TODO: All this should be done dynamically
// We could store the disk size in the superblock area
// And we could maybe also have a counter of allocated block in there to make
// disk usage report O(1)
const BLOCK_SIZE: usize = 512;
const DISK_SIZE: usize = (8 << 20) / BLOCK_SIZE; // 8 MB disk
const KERNEL_SIZE: usize = (2 << 20) / BLOCK_SIZE; // 2 MB for the kernel binary
const MAX_BLOCKS: usize = (DISK_SIZE - KERNEL_SIZE) / 2; // FIXME: Replace `/ 2` with `- SUPELBLOCK_AREA_SIZE - BITMAP_AREA_SIZE`
const SUPERBLOCK_ADDR: u32 = KERNEL_SIZE as u32; // Address of the block
const BITMAP_ADDR: u32 = SUPERBLOCK_ADDR + 2;
const DATA_ADDR: u32 = BITMAP_ADDR + ((MAX_BLOCKS as u32) / block_bitmap::BITMAP_SIZE as u32 / 8); // 1 bit per block in bitmap

pub fn disk_size() -> usize {
    DISK_SIZE * BLOCK_SIZE
}

// FIXME: this should be BLOCK_SIZE times faster
pub fn disk_used() -> usize {
    let mut used_blocks_count = 0;
    let n = MAX_BLOCKS as u32;
    for i in 0..n {
        let addr = DATA_ADDR + i;
        if BlockBitmap::is_alloc(addr) {
            used_blocks_count += 1;
        }
    }
    used_blocks_count * BLOCK_SIZE
}

pub fn disk_free() -> usize {
    disk_size() - disk_used()
}

pub fn init() {
    /*
    printk!("disk size       = {} blocks\n", DISK_SIZE);
    printk!("kernel size     = {} blocks\n", KERNEL_SIZE);
    printk!("superblock addr = {}\n", SUPERBLOCK_ADDR);
    printk!("bitmap addr     = {}\n", BITMAP_ADDR);
    printk!("data addr       = {}\n", DATA_ADDR);
    printk!("end addr        = {}\n", DATA_ADDR + MAX_BLOCKS as u32);
    */

    block_device::init();
}
