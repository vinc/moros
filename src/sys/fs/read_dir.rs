use super::dir_entry::DirEntry;
use super::block::LinkedBlock;
use super::dir::Dir;
use super::FileType;

use alloc::string::String;
use core::convert::From;
use core::convert::TryInto;

pub struct ReadDir {
    // TODO: make those fields private
    pub dir: Dir,
    pub block: LinkedBlock,
    pub block_offset: usize,
    block_index: usize,
}

impl From<Dir> for ReadDir {
    fn from(dir: Dir) -> Self {
        Self {
            dir: dir.clone(),
            block: LinkedBlock::read(dir.addr()),
            block_offset: 0,
            block_index: 0,
        }
    }
}

macro_rules! read_uint_fn {
    ($name:ident, $type:ident) => {
        fn $name(&mut self) -> $type {
            let data = self.block.data();
            let a = self.block_offset;
            let b = a + core::mem::size_of::<$type>();
            self.block_offset = b;
            $type::from_be_bytes(data[a..b].try_into().unwrap())
        }
    };
}

impl ReadDir {
    /// Total number of bytes read
    pub fn offset(&self) -> usize {
        self.block_index * self.block.len() + self.block_offset
    }

    /// Number of bytes read in current block
    pub fn block_offset(&self) -> usize {
        self.block_offset
    }

    /// Address of current block
    pub fn block_addr(&self) -> u32 {
        self.block.addr()
    }

    read_uint_fn!(read_u8, u8);
    read_uint_fn!(read_u32, u32);
    read_uint_fn!(read_u64, u64);

    fn read_utf8_lossy(&mut self, len: usize) -> String {
        let data = self.block.data();
        let a = self.block_offset;
        let b = a + len;
        self.block_offset = b;
        String::from_utf8_lossy(&data[a..b]).into()
    }
}

impl Iterator for ReadDir {
    type Item = DirEntry;

    fn next(&mut self) -> Option<DirEntry> {
        loop {
            loop {
                let offset = self.block_offset; // Backup cursor position

                // Switch to next block if no space left for another entry
                if offset >= self.block.len() - DirEntry::empty_len() {
                    break;
                }

                let entry_kind = match self.read_u8() {
                    0 => FileType::Dir,
                    1 => FileType::File,
                    2 => FileType::Device,
                    _ => {
                        self.block_offset = offset; // Rewind the cursor
                        break;
                    },
                };

                let entry_addr = self.read_u32();
                let entry_size = self.read_u32();
                let entry_time = self.read_u64();

                let n = self.read_u8() as usize;
                if n == 0 || n >= self.block.len() - self.block_offset {
                    self.block_offset = offset; // Rewind the cursor
                    break;
                }

                // The rest of the entry is the filename string
                let entry_name = self.read_utf8_lossy(n);

                // Skip deleted entries
                if entry_addr == 0 {
                    continue;
                }

                let dir = self.dir.clone();
                return Some(DirEntry::new(dir, entry_kind, entry_addr, entry_size, entry_time, &entry_name));
            }

            match self.block.next() {
                Some(next_block) => {
                    self.block = next_block;
                    self.block_offset = 0;
                    self.block_index += 1;
                }
                None => break,
            }
        }

        None
    }
}
