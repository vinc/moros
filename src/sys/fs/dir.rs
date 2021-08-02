use super::data_addr;
use super::{dirname, filename, realpath};
use super::dir_entry::DirEntry;
use super::read_dir::ReadDir;
use super::block_bitmap::BlockBitmap;
use super::FileType;
use super::block::Block;

use bit_field::BitField;
use core::convert::From;
use crate::sys;

#[derive(Clone, Copy)]
pub struct Dir {
    addr: u32,
}

impl From<DirEntry> for Dir {
    fn from(entry: DirEntry) -> Self {
        Self { addr: entry.addr() }
    }
}

impl Dir {
    pub fn root() -> Self {
        Self { addr: data_addr() }
    }

    pub fn create(pathname: &str) -> Option<Self> {
        let pathname = realpath(pathname);
        let dirname = dirname(&pathname);
        let filename = filename(&pathname);
        if let Some(dir) = Dir::open(dirname) {
            if let Some(dir_entry) = dir.create_dir(filename) {
                return Some(dir_entry.into());
            }
        }
        None
    }

    pub fn open(pathname: &str) -> Option<Self> {
        let pathname = realpath(pathname);
        let mut dir = Dir::root();

        if !super::is_mounted() {
            return None;
        }

        if pathname == "/" {
            return Some(dir);
        }

        for name in pathname.trim_start_matches('/').split('/') {
            match dir.find(name) {
                Some(dir_entry) => {
                    if dir_entry.is_dir() {
                        dir = dir_entry.into()
                    } else {
                        return None;
                    }
                },
                None => {
                    return None
                },
            }
        }
        Some(dir)
    }

    pub fn addr(&self) -> u32 {
        self.addr
    }

    pub fn find(&self, name: &str) -> Option<DirEntry> {
        for entry in self.read() {
            if entry.name() == name {
                return Some(entry);
            }
        }
        None
    }

    pub fn create_file(&self, name: &str) -> Option<DirEntry> {
        self.create_entry(FileType::File, name)
    }

    pub fn create_dir(&self, name: &str) -> Option<DirEntry> {
        self.create_entry(FileType::Dir, name)
    }

    fn create_entry(&self, kind: FileType, name: &str) -> Option<DirEntry> {
        if self.find(name).is_some() {
            return None;
        }

        let mut read_dir = self.read();
        while read_dir.next().is_some() {}
        //let block = read_dir.block();
        //let block_data_offset = read_dir.block_data_offset();

        // Allocate a new block for the dir if no space left for adding the new entry
        // TODO: move that somewhere
        let space_left = read_dir.block.data().len() - read_dir.block_data_offset();
        let entry_len = DirEntry::empty_len() + name.len();
        if entry_len > space_left {
            let new_block = Block::alloc().unwrap();
            read_dir.block.set_next(new_block.addr());
            read_dir.block.write();
            read_dir.block = new_block;
            read_dir.block_data_offset = 0;
        }

        // Create a new entry
        // TODO: add DirEntry::create()
        let entry_block = Block::alloc().unwrap();
        let entry_addr = entry_block.addr();
        let entry_kind = kind;
        let entry_size = 0;
        let entry_time = sys::clock::realtime() as u64;
        let entry_name = name.as_bytes();

        let n = entry_name.len();
        let i = read_dir.block_data_offset();
        let data = read_dir.block.data_mut();
        data[i +  0] = entry_kind as u8;
        data[i +  1] = entry_addr.get_bits(24..32) as u8;
        data[i +  2] = entry_addr.get_bits(16..24) as u8;
        data[i +  3] = entry_addr.get_bits(8..16) as u8;
        data[i +  4] = entry_addr.get_bits(0..8) as u8;
        data[i +  5] = entry_size.get_bits(24..32) as u8;
        data[i +  6] = entry_size.get_bits(16..24) as u8;
        data[i +  7] = entry_size.get_bits(8..16) as u8;
        data[i +  8] = entry_size.get_bits(0..8) as u8;
        data[i +  9] = entry_time.get_bits(56..64) as u8;
        data[i + 10] = entry_time.get_bits(48..56) as u8;
        data[i + 11] = entry_time.get_bits(40..48) as u8;
        data[i + 12] = entry_time.get_bits(32..40) as u8;
        data[i + 13] = entry_time.get_bits(24..32) as u8;
        data[i + 14] = entry_time.get_bits(16..24) as u8;
        data[i + 15] = entry_time.get_bits(8..16) as u8;
        data[i + 16] = entry_time.get_bits(0..8) as u8;
        data[i + 17] = n as u8;
        for j in 0..n {
            data[i + 18 + j] = entry_name[j];
        }
        read_dir.block.write();

        Some(DirEntry::new(*self, kind, entry_addr, entry_size, entry_time, name))
    }

    // Deleting an entry is done by setting the entry address to 0
    // TODO: If the entry is a directory, remove its entries recursively
    pub fn delete_entry(&mut self, name: &str) -> Result<(), ()> {
        let mut read_dir = self.read();
        for entry in &mut read_dir {
            if entry.name() == name {
                // Zeroing entry addr
                let i = read_dir.block_data_offset() - entry.len();
                let data = read_dir.block.data_mut();
                data[i + 1] = 0;
                data[i + 2] = 0;
                data[i + 3] = 0;
                data[i + 4] = 0;
                read_dir.block.write();

                // Freeing entry blocks
                let mut entry_block = Block::read(entry.addr());
                loop {
                    BlockBitmap::free(entry_block.addr());
                    match entry_block.next() {
                        Some(next_block) => entry_block = next_block,
                        None => break,
                    }
                }

                return Ok(());
            }
        }
        Err(())
    }

    pub fn update_entry(&mut self, name: &str, size: u32) {
        let mut read_dir = self.read();
        for entry in &mut read_dir {
            if entry.name() == name {
                let time = sys::clock::realtime() as u64;
                let i = read_dir.block_data_offset() - entry.len();
                let data = read_dir.block.data_mut();
                data[i +  5] = size.get_bits(24..32) as u8;
                data[i +  6] = size.get_bits(16..24) as u8;
                data[i +  7] = size.get_bits(8..16) as u8;
                data[i +  8] = size.get_bits(0..8) as u8;
                data[i +  9] = time.get_bits(56..64) as u8;
                data[i + 10] = time.get_bits(48..56) as u8;
                data[i + 11] = time.get_bits(40..48) as u8;
                data[i + 12] = time.get_bits(32..40) as u8;
                data[i + 13] = time.get_bits(24..32) as u8;
                data[i + 14] = time.get_bits(16..24) as u8;
                data[i + 15] = time.get_bits(8..16) as u8;
                data[i + 16] = time.get_bits(0..8) as u8;
                read_dir.block.write();
                break;
            }
        }
    }

    pub fn read(&self) -> ReadDir {
        ReadDir::from(self.clone())
    }

    pub fn delete(pathname: &str) -> Result<(), ()> {
        let pathname = realpath(pathname);
        let dirname = dirname(&pathname);
        let filename = filename(&pathname);
        if let Some(mut dir) = Dir::open(dirname) {
            dir.delete_entry(filename)
        } else {
            Err(())
        }
    }
}

