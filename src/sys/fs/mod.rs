mod block;
mod bitmap_block;
mod block_device;
mod device;
mod dir;
mod dir_entry;
mod file;
mod read_dir;
mod super_block;

use crate::sys;

pub use bitmap_block::BITMAP_SIZE;
pub use device::{Device, DeviceType};
pub use dir::Dir;
pub use dir_entry::FileStat;
pub use file::{File, SeekFrom};
pub use block_device::{format_ata, format_mem, is_mounted, mount_ata, mount_mem, dismount};
pub use crate::api::fs::{dirname, filename, realpath, FileIO};
pub use crate::sys::ata::BLOCK_SIZE;

use dir_entry::DirEntry;
use super_block::SuperBlock;

use alloc::string::{String, ToString};

pub const VERSION: u8 = 1;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum OpenFlag {
    Read   = 1,
    Write  = 2,
    Create = 4,
    Dir    = 8,
    Device = 16,
}

impl OpenFlag {
    fn is_set(&self, flags: usize) -> bool {
        flags & (*self as usize) != 0
    }
}

pub fn open(path: &str, flags: usize) -> Option<Resource> {
    if OpenFlag::Dir.is_set(flags) {
        let res = Dir::open(path);
        if res.is_none() && OpenFlag::Create.is_set(flags) {
            Dir::create(path)
        } else {
            res
        }.map(Resource::Dir)
    } else if OpenFlag::Device.is_set(flags) {
        let res = Device::open(path);
        if res.is_none() && OpenFlag::Create.is_set(flags) {
            Device::create(path)
        } else {
            res
        }.map(Resource::Device)
    } else {
        let res = File::open(path);
        if res.is_none() && OpenFlag::Create.is_set(flags) {
            File::create(path)
        } else {
            res
        }.map(Resource::File)
    }
}

pub fn delete(path: &str) -> Result<(), ()> {
    if let Some(stat) = stat(path) {
        if stat.is_file() {
            return File::delete(path);
        } else if stat.is_dir() {
            return Dir::delete(path);
        }
    }
    Err(())
}

pub fn stat(pathname: &str) -> Option<FileStat> {
    if pathname == "/" {
        return Some(FileStat::root());
    }
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

pub fn canonicalize(path: &str) -> Result<String, ()> {
    match sys::process::env("HOME") {
        Some(home) => {
            if path.starts_with('~') {
                Ok(path.replace('~', &home))
            } else {
                Ok(path.to_string())
            }
        },
        None => {
            Ok(path.to_string())
        }
    }
}

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
    for bus in 0..2 {
        for dsk in 0..2 {
            if SuperBlock::check_ata(bus, dsk) {
                log!("MFS Superblock found in ATA {}:{}\n", bus, dsk);
                mount_ata(bus, dsk);
                return;
            }
        }
    }
}
