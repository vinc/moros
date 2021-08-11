use super::{dirname, filename, realpath};
use super::dir::Dir;
use super::block::Block;
use super::dir_entry::DirEntry;

use alloc::string::{String, ToString};
use alloc::vec;
use core::convert::From;

pub enum SeekFrom {
    Start(u32),
    Current(i32),
    End(i32),
}

#[derive(Clone)]
pub struct File {
    name: String,
    addr: u32,
    size: u32,
    time: u64,
    dir: Dir, // TODO: Replace with `parent: Some(Dir)` and also add it to `Dir`
    offset: u32,
}

impl From<DirEntry> for File {
    fn from(entry: DirEntry) -> Self {
        Self {
            name: entry.name(),
            addr: entry.addr(),
            size: entry.size(),
            time: entry.time(),
            dir: entry.dir(),
            offset: 0,
        }
    }
}

impl File {
    pub fn create(pathname: &str) -> Option<Self> {
        let pathname = realpath(pathname);
        let dirname = dirname(&pathname);
        let filename = filename(&pathname);
        if let Some(dir) = Dir::open(dirname) {
            if let Some(dir_entry) = dir.create_file(filename) {
                return Some(dir_entry.into());
            }
        }
        None
    }

    pub fn open(pathname: &str) -> Option<Self> {
        let pathname = realpath(pathname);
        let dirname = dirname(&pathname);
        let filename = filename(&pathname);
        if let Some(dir) = Dir::open(dirname) {
            if let Some(dir_entry) = dir.find(filename) {
                if dir_entry.is_file() {
                    return Some(dir_entry.into());
                }
            }
        }
        None
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }

    pub fn seek(&mut self, pos: SeekFrom) -> Result<u32, ()> {
        let offset = match pos {
            SeekFrom::Start(i)   => i as i32,
            SeekFrom::Current(i) => i + self.offset as i32,
            SeekFrom::End(i)     => i + self.size as i32 - 1,
        };
        if offset < 0 || offset > self.size as i32 { // TODO: offset > size?
            return Err(())
        }
        self.offset = offset as u32;

        Ok(self.offset)
    }

    // TODO: return `Result<usize>`
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let buf_len = buf.len();
        let mut addr = self.addr;
        let mut bytes = 0; // Number of bytes read
        let mut pos = 0; // Position in the file
        loop {
            let block = Block::read(addr);
            let data = block.data();
            let data_len = data.len();
            for i in 0..data_len {
                if pos == self.offset {
                    if bytes == buf_len || pos as usize == self.size() {
                        return bytes;
                    }
                    buf[bytes] = data[i];
                    bytes += 1;
                    self.offset += 1;
                }
                pos += 1;
            }
            match block.next() {
                Some(next_block) => addr = next_block.addr(),
                None => return bytes,
            }
        }
    }

    // TODO: add `read_to_end(&self, buf: &mut Vec<u8>) -> Result<u32>`

    // TODO: `return Result<String>`
    pub fn read_to_string(&mut self) -> String {
        let mut buf = vec![0; self.size()];
        let bytes = self.read(&mut buf);
        buf.resize(bytes, 0);
        String::from_utf8_lossy(&buf).to_string()
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        let buf_len = buf.len();
        let mut addr = self.addr;
        let mut bytes = 0; // Number of bytes written
        let mut pos = 0; // Position in the file
        while bytes < buf_len {
            let mut block = Block::read(addr);
            let data = block.data_mut();
            let data_len = data.len();
            for i in 0..data_len {
                if pos == self.offset {
                    if bytes == buf_len {
                        break;
                    }
                    data[i] = buf[bytes];
                    bytes += 1;
                    self.offset += 1;
                }
                pos += 1;
            }

            addr = match block.next() {
                Some(next_block) => {
                    if bytes < buf_len {
                        next_block.addr()
                    } else {
                        // TODO: Free the next block(s)
                        0
                    }
                }
                None => {
                    if bytes < buf_len {
                        match Block::alloc() {
                            Some(next_block) => next_block.addr(),
                            None => return Err(()),
                        }
                    } else {
                        0
                    }
                }
            };

            block.set_next_addr(addr);
            block.write();
        }
        self.size = self.offset;
        self.dir.update_entry(&self.name, self.size);
        Ok(bytes)
    }

    pub fn addr(&self) -> u32 {
        self.addr
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

#[test_case]
fn test_file_create() {
    super::mount_mem();
    super::format_mem();
    assert!(File::create("/test").is_some());
    assert_eq!(File::create("/hello").unwrap().name(), "hello");
    super::dismount();
}

#[test_case]
fn test_file_write() {
    super::mount_mem();
    super::format_mem();
    let mut file = File::create("/test").unwrap();
    let buf = "Hello, World!".as_bytes();
    assert_eq!(file.write(&buf), Ok(buf.len()));
    super::dismount();
}

#[test_case]
fn test_file_open() {
    super::mount_mem();
    super::format_mem();
    assert!(File::open("/test").is_none());
    let mut file = File::create("/test").unwrap();
    let buf = "Hello, World!".as_bytes();
    file.write(&buf).unwrap();
    assert!(File::open("/test").is_some());
    super::dismount();
}

#[test_case]
fn test_file_read() {
    super::mount_mem();
    super::format_mem();
    let mut file = File::create("/test").unwrap();
    let input = "Hello, World!".as_bytes();
    file.write(&input).unwrap();

    let mut file = File::open("/test").unwrap();
    let mut output = [0u8; 13];
    assert_eq!(file.read(&mut output), input.len());
    assert_eq!(input, output);
    super::dismount();
}

#[test_case]
fn test_file_delete() {
    super::mount_mem();
    super::format_mem();
    assert!(File::open("/test").is_none());
    assert!(File::create("/test").is_some());
    assert!(File::open("/test").is_some());
    assert!(File::delete("/test").is_ok());
    assert!(File::open("/test").is_none());
    super::dismount();
}
