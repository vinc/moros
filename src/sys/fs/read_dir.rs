use super::dir_entry::DirEntry;
use super::block::Block;
use super::dir::Dir;
use super::FileType;

use alloc::string::String;
use core::convert::From;

pub struct ReadDir {
    // TODO: make those fields private
    pub dir: Dir,
    pub block: Block,
    pub block_data_offset: usize,
}

impl From<Dir> for ReadDir {
    fn from(dir: Dir) -> Self {
        Self {
            dir: dir,
            block: Block::read(dir.addr()),
            block_data_offset: 0,
        }
    }
}

impl ReadDir {
    pub fn block_data_offset(&self) -> usize {
        self.block_data_offset
    }

    pub fn block_addr(&self) -> u32 {
        self.block.addr()
    }
}

impl Iterator for ReadDir {
    type Item = DirEntry;

    fn next(&mut self) -> Option<DirEntry> {
        loop {
            let data = self.block.data();
            let mut i = self.block_data_offset;

            loop {
                // Switch to next block if no space left for another entry
                if i == data.len() - DirEntry::empty_len() {
                    break;
                }

                let entry_kind = match data[i + 0] {
                    0 => FileType::Dir,
                    1 => FileType::File,
                    _ => break,
                };

                let entry_addr = (data[i +  1] as u32) << 24
                               | (data[i +  2] as u32) << 16
                               | (data[i +  3] as u32) << 8
                               | (data[i +  4] as u32);

                let entry_size = (data[i +  5] as u32) << 24
                               | (data[i +  6] as u32) << 16
                               | (data[i +  7] as u32) << 8
                               | (data[i +  8] as u32);

                let entry_time = (data[i +  9] as u64) << 56
                               | (data[i + 10] as u64) << 48
                               | (data[i + 11] as u64) << 40
                               | (data[i + 12] as u64) << 32
                               | (data[i + 13] as u64) << 24
                               | (data[i + 14] as u64) << 16
                               | (data[i + 15] as u64) << 8
                               | (data[i + 16] as u64);
                i += 17;

                let mut n = data[i];
                if n == 0 || n as usize >= data.len() - i {
                    break;
                }
                i += 1;

                // The rest of the entry is the pathname string.
                let mut entry_name = String::new();
                loop {
                    if n == 0 {
                        break;
                    }
                    entry_name.push(data[i] as char);
                    n -= 1;
                    i += 1;
                }

                self.block_data_offset = i;

                // Skip deleted entries
                if entry_addr == 0 {
                    continue;
                }

                return Some(DirEntry::new(self.dir, entry_kind, entry_addr, entry_size, entry_time, &entry_name));
            }

            match self.block.next() {
                Some(next_block) => {
                    self.block = next_block;
                    self.block_data_offset = 0;
                }
                None => break,
            }
        }

        None
    }
}
