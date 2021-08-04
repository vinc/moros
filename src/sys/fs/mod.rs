mod block;
mod block_bitmap;
mod block_device;
mod dir;
mod dir_entry;
mod file;
mod read_dir;

pub use dir::Dir;
pub use file::{File, SeekFrom};
pub use block_device::{format, is_mounted, mount};

use crate::sys;
use alloc::format;
use alloc::string::String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Dir = 0,
    File = 1,
}

pub fn dirname(pathname: &str) -> &str {
    let n = pathname.len();
    let i = match pathname.rfind('/') {
        Some(0) => 1,
        Some(i) => i,
        None => n,
    };
    &pathname[0..i]
}

pub fn filename(pathname: &str) -> &str {
    let n = pathname.len();
    let i = match pathname.rfind('/') {
        Some(i) => i + 1,
        None => 0,
    };
    &pathname[i..n]
}

// Transform "foo.txt" into "/path/to/foo.txt"
pub fn realpath(pathname: &str) -> String {
    if pathname.starts_with('/') {
        pathname.into()
    } else {
        let dirname = sys::process::dir();
        let sep = if dirname.ends_with('/') { "" } else { "/" };
        format!("{}{}{}", dirname, sep, pathname)
    }
}

const BLOCK_SIZE: usize = 512;
const MAX_BLOCKS: usize = 2 * 2048;

const DISK_OFFSET: u32 = 4 << 10; // Leave space for kernel binary
const SUPERBLOCK_ADDR: u32 = DISK_OFFSET;
const BITMAP_ADDR: u32 = DISK_OFFSET + 2;
const DATA_ADDR: u32 = BITMAP_ADDR + (MAX_BLOCKS as u32) / 8;

pub const fn block_size() -> usize {
    BLOCK_SIZE
}

pub const fn max_block() -> usize {
    MAX_BLOCKS
}

pub const fn superblock_addr() -> u32 {
    SUPERBLOCK_ADDR
}

pub const fn data_addr() -> u32 {
    DATA_ADDR
}

pub const fn bitmap_addr() -> u32 {
    BITMAP_ADDR
}

/* Disk Areas
 * 1 => Reserved
 * 2 => Bitmap (allocated blocks (1 bit per block)
 * 3 => Data (directories and files)
 */

pub fn init() {
    block_device::init();
}
