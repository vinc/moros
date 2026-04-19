use super::bitmap_block::BitmapBlock;
use super::block::LinkedBlock;
use super::dir_entry::DirEntry;
use super::read_dir::ReadDir;
use super::super_block::SuperBlock;
use super::FileType;
use super::{dirname, filename, realpath, FileIO, IO};
use crate::sys;

use alloc::boxed::Box;
use alloc::string::String;

#[derive(Debug, Clone)]
pub struct Dir {
    parent: Option<Box<Dir>>,
    name: String,
    addr: u32,
    size: u32,
    entry_index: u32,
}

impl From<DirEntry> for Dir {
    fn from(entry: DirEntry) -> Self {
        Self {
            parent: Some(Box::new(entry.dir())),
            name: entry.name(),
            addr: entry.addr(),
            size: entry.size(),
            entry_index: 0,
        }
    }
}

impl Dir {
    pub fn root() -> Self {
        let name = String::new();
        let addr = SuperBlock::read().data_area();
        let mut root = Self {
            parent: None,
            name,
            addr,
            size: 0,
            entry_index: 0,
        };
        root.update_size();
        root
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn create(pathname: &str) -> Option<Self> {
        let pathname = realpath(pathname);
        let dirname = dirname(&pathname);
        let filename = filename(&pathname);
        if let Some(mut dir) = Dir::open(dirname) {
            if let Some(dir_entry) = dir.create_dir(filename) {
                return Some(dir_entry.into());
            }
        }
        None
    }

    pub fn open(pathname: &str) -> Option<Self> {
        if !super::is_mounted() {
            return None;
        }

        let mut dir = Dir::root();
        let pathname = realpath(pathname);

        if pathname == "/" {
            return Some(dir);
        }

        for name in pathname.trim_start_matches('/').split('/') {
            match dir.find(name) {
                Some(entry) if entry.is_dir() => dir = entry.into(),
                _ => return None,
            }
        }
        Some(dir)
    }

    pub fn addr(&self) -> u32 {
        self.addr
    }

    pub fn find(&self, name: &str) -> Option<DirEntry> {
        self.entries().find(|entry| entry.name() == name)
    }

    // TODO: return a Result
    pub fn create_file(&mut self, name: &str) -> Option<DirEntry> {
        self.create_entry(FileType::File, name)
    }

    pub fn create_dir(&mut self, name: &str) -> Option<DirEntry> {
        self.create_entry(FileType::Dir, name)
    }

    pub fn create_device(&mut self, name: &str) -> Option<DirEntry> {
        self.create_entry(FileType::Device, name)
    }

    fn create_entry(&mut self, kind: FileType, name: &str) -> Option<DirEntry> {
        if self.find(name).is_some() {
            return None;
        }

        let mut space_found = false;
        let entry_len = DirEntry::empty_len() + name.len();

        // Read the whole dir to find where to write the new entry
        let mut entries = self.entries();
        entries.skip_unused = false;
        while let Some(other) = entries.next() {
            // A unused dir entry is a virtual entry with a null addr
            // and a size indicating the unused space
            if other.addr() == 0 {
                // Bytes read for the entry
                let read = DirEntry::empty_len() + other.name().len();

                // Bytes from the start of the entry to the end of the block
                let rest = entries.block.len() - entries.block_offset + read;

                if entry_len > rest {
                    continue; // Not enough space
                }

                // Check if the unused space goes to the end of the block or
                // of if it fit exactly the new entry in the case of a deleted
                // entry from previous versions of MOROS
                let other_len = other.size() as usize;
                if other_len == rest || other_len == entry_len {
                    space_found = true;
                    entries.block_offset -= read; // Rewind to write over
                    break;
                }
            }
        }

        // Allocate a new block for the dir if there's no space left for adding
        // the new entry
        let space_left = entries.block.data().len() - entries.block_offset();
        if entry_len > space_left && !space_found {
            match entries.block.alloc_next() {
                None => return None, // Disk is full
                Some(block) => {
                    entries.block = block;
                    entries.block_offset = 0;
                }
            }
        }

        // Create a new entry
        let entry_block = LinkedBlock::alloc()?;
        let entry_kind = kind as u8;
        let entry_addr = entry_block.addr();
        let entry_size = 0u32;
        let entry_time = sys::clk::epoch_time() as u64;
        let entry_name = truncate(name, u8::MAX as usize);
        let n = entry_name.len();
        let i = entries.block_offset();
        let data = entries.block.data_mut();

        data[i] = entry_kind;
        data[(i + 1)..(i + 5)].clone_from_slice(&entry_addr.to_be_bytes());
        data[(i + 5)..(i + 9)].clone_from_slice(&entry_size.to_be_bytes());
        data[(i + 9)..(i + 17)].clone_from_slice(&entry_time.to_be_bytes());
        data[i + 17] = n as u8;
        data[(i + 18)..(i + 18 + n)].clone_from_slice(entry_name.as_bytes());

        entries.block.write();
        self.update_size();

        Some(DirEntry::new(
            self.clone(),
            kind,
            entry_addr,
            entry_size,
            entry_time,
            &entry_name,
        ))
    }

    // NOTE: Directories must be empty before deletion
    pub fn delete_entry(&mut self, name: &str) -> Result<(), ()> {
        let mut entries = self.entries();
        let mut last_block_addr = 0;
        let mut this_block_addr = 0;
        while let Some(entry) = entries.next() {
            if entries.block.addr() != this_block_addr {
                last_block_addr = this_block_addr;
                this_block_addr = entries.block.addr();
            }
            if entry.name() == name {
                let i = entries.block_offset() - entry.len();
                let j = entries.block.len() - entry.len();
                let reminder = entries.block_offset()..entries.block.len();
                let data = entries.block.data_mut();

                // Shift the reminder of the block over this entry
                data.copy_within(reminder, i);

                // Clear the unused end of the block
                data[j..].fill(0);

                entries.block.write();

                // Free empty dir block (except the first one)
                if entries.block.is_empty() && last_block_addr != 0 {
                    let mut prev_block = LinkedBlock::read(last_block_addr);
                    prev_block.set_next_addr(entries.block.next_addr());
                    prev_block.write();
                    BitmapBlock::free(entries.block.addr());
                }

                // Free entry blocks
                let mut free_block = LinkedBlock::read(entry.addr());
                loop {
                    BitmapBlock::free(free_block.addr());
                    match free_block.next() { // FIXME: read after free?
                        Some(next_block) => free_block = next_block,
                        None => break,
                    }
                }

                self.update_size();

                return Ok(());
            }
        }

        Err(())
    }

    pub fn update_entry(&self, name: &str, size: u32) {
        let time = sys::clk::epoch_time() as u64;
        let mut entries = self.entries();
        for entry in &mut entries {
            if entry.name() == name {
                let i = entries.block_offset() - entry.len();
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

    pub fn size(&self) -> usize {
        self.size as usize
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

    fn update_size(&mut self) {
        // The size of a dir is the sum of its dir entries
        let size: usize = self.entries().map(|e| e.len()).sum();
        self.size = size as u32;
        if let Some(dir) = self.parent.clone() {
            dir.update_entry(&self.name, self.size);
        }
    }
}

impl FileIO for Dir {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let mut i = 0;
        for entry in self.entries().skip(self.entry_index as usize) {
            let info = entry.info();
            let bytes = info.as_bytes();
            let j = i + bytes.len();
            if j < buf.len() {
                buf[i..j].copy_from_slice(&bytes);
                self.entry_index += 1;
                i = j;
            } else {
                break;
            }
        }
        Ok(i)
    }

    fn write(&mut self, _buf: &[u8]) -> Result<usize, ()> {
        Err(())
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => self.entry_index < self.entries().count() as u32,
            IO::Write => true,
        }
    }
}

// Truncate to the given number of bytes at most
// while respecting char boundaries.
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
