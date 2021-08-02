use super::dir_entry::DirEntry;
use super::block::Block;
use super::dir::Dir;
use super::FileType;

use alloc::string::String;

pub struct ReadDir {
    pub dir: Dir,
    pub block: Block,
    pub data_offset: usize,
}

impl Iterator for ReadDir {
    type Item = DirEntry;

    fn next(&mut self) -> Option<DirEntry> {
        loop {
            let data = self.block.data();
            let mut i = self.data_offset;

            loop {
                if i == data.len() - 10 { // No space left for another entry in the block
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

                self.data_offset = i;

                // Skip deleted entries
                if entry_addr == 0 {
                    continue;
                }

                return Some(DirEntry::new(self.dir, entry_kind, entry_addr, entry_size, entry_time, &entry_name));
            }

            match self.block.next() {
                Some(next_block) => {
                    self.block = next_block;
                    self.data_offset = 0;
                }
                None => break,
            }
        }

        None
    }
}
