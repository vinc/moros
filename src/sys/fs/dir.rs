use super::{dirname, filename, realpath, FileIO};
use super::dir_entry::DirEntry;
use super::read_dir::ReadDir;
use super::block_bitmap::BlockBitmap;
use super::FileType;
use super::block::Block;
use crate::sys;

use alloc::string::String;
use core::convert::From;

#[derive(Debug, Clone, Copy)]
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
        Self { addr: super::DATA_ADDR }
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
        for entry in self.entries() {
            if entry.name() == name {
                return Some(entry);
            }
        }
        None
    }

    // TODO: return a Result
    pub fn create_file(&self, name: &str) -> Option<DirEntry> {
        self.create_entry(FileType::File, name)
    }

    pub fn create_dir(&self, name: &str) -> Option<DirEntry> {
        self.create_entry(FileType::Dir, name)
    }

    pub fn create_device(&self, name: &str) -> Option<DirEntry> {
        self.create_entry(FileType::Device, name)
    }

    fn create_entry(&self, kind: FileType, name: &str) -> Option<DirEntry> {
        if self.find(name).is_some() {
            return None;
        }

        // Read the whole dir to add an entry at the end
        let mut entries = self.entries();
        while entries.next().is_some() {}

        // Allocate a new block for the dir if no space left for adding the new entry
        let space_left = entries.block.data().len() - entries.block_data_offset();
        let entry_len = DirEntry::empty_len() + name.len();
        if entry_len > space_left {
            match entries.block.alloc_next() {
                None => return None, // Disk is full
                Some(new_block) => {
                    entries.block = new_block;
                    entries.block_data_offset = 0;
                },
            }
        }

        // Create a new entry
        let entry_block = Block::alloc().unwrap();
        let entry_kind = kind as u8;
        let entry_addr = entry_block.addr();
        let entry_size = 0u32;
        let entry_time = sys::clock::realtime() as u64;
        let entry_name = truncate(name, u8::MAX as usize);
        let n = entry_name.len();
        let i = entries.block_data_offset();
        let data = entries.block.data_mut();

        data[i] = entry_kind;
        data[(i + 1)..(i + 5)].clone_from_slice(&entry_addr.to_be_bytes());
        data[(i + 5)..(i + 9)].clone_from_slice(&entry_size.to_be_bytes());
        data[(i + 9)..(i + 17)].clone_from_slice(&entry_time.to_be_bytes());
        data[i + 17] = n as u8;
        data[(i + 18)..(i + 18 + n)].clone_from_slice(&entry_name.as_bytes());

        entries.block.write();

        Some(DirEntry::new(*self, kind, entry_addr, entry_size, entry_time, &entry_name))
    }

    // Deleting an entry is done by setting the entry address to 0
    // TODO: If the entry is a directory, remove its entries recursively
    pub fn delete_entry(&mut self, name: &str) -> Result<(), ()> {
        let mut entries = self.entries();
        for entry in &mut entries {
            if entry.name() == name {
                // Zeroing entry addr
                let i = entries.block_data_offset() - entry.len();
                let data = entries.block.data_mut();
                data[i + 1] = 0;
                data[i + 2] = 0;
                data[i + 3] = 0;
                data[i + 4] = 0;
                entries.block.write();

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
        let time = sys::clock::realtime() as u64;
        let mut entries = self.entries();
        for entry in &mut entries {
            if entry.name() == name {
                let i = entries.block_data_offset() - entry.len();
                let data = entries.block.data_mut();
                data[(i + 5)..(i + 9)].clone_from_slice(&size.to_be_bytes());
                data[(i + 9)..(i + 17)].clone_from_slice(&time.to_be_bytes());
                entries.block.write();
                break;
            }
        }
    }

    pub fn entries(&self) -> ReadDir {
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

impl FileIO for Dir {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, ()> {
        Err(())
    }
    fn write(&mut self, _buf: &[u8]) -> Result<usize, ()> {
        Err(())
    }
}

// Truncate to the given number of bytes at most while respecting char boundaries
fn truncate(s: &str, max: usize) -> String {
    s.char_indices().take_while(|(i, _)| *i <= max).map(|(_, c)| c).collect()
}

#[test_case]
fn test_dir_create() {
    super::mount_mem();
    super::format_mem();
    assert!(Dir::open("/test").is_none());
    assert!(Dir::create("/test").is_some());
    assert!(Dir::open("/test").is_some());

    assert!(Dir::open("/test/test").is_none());
    assert!(Dir::create("/test/test").is_some());
    assert!(Dir::open("/test/test").is_some());
    super::dismount();
}

#[test_case]
fn test_dir_delete() {
    super::mount_mem();
    super::format_mem();
    assert!(Dir::open("/test").is_none());
    assert!(Dir::create("/test").is_some());
    assert!(Dir::open("/test").is_some());
    assert!(Dir::delete("/test").is_ok());
    assert!(Dir::open("/test").is_none());
    super::dismount();
}
