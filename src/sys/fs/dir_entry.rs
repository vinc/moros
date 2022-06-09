use super::{dirname, filename, realpath, FileType};
use super::dir::Dir;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone)]
pub struct DirEntry {
    dir: Dir,
    addr: u32,

    // FileInfo
    kind: FileType,
    size: u32,
    time: u64,
    name: String,
}

impl DirEntry {
    pub fn open(pathname: &str) -> Option<Self> {
        let pathname = realpath(pathname);
        let dirname = dirname(&pathname);
        let filename = filename(&pathname);
        if let Some(dir) = Dir::open(dirname) {
            return dir.find(filename);
        }
        None
    }

    pub fn new(dir: Dir, kind: FileType, addr: u32, size: u32, time: u64, name: &str) -> Self {
        let name = String::from(name);
        Self { dir, kind, addr, size, time, name }
    }

    pub fn empty_len() -> usize {
        1 + 4 + 4 + 8 + 1
    }

    pub fn len(&self) -> usize {
        Self::empty_len() + self.name.len()
    }

    pub fn is_empty(&self) -> bool {
        Self::empty_len() == self.len()
    }

    pub fn kind(&self) -> FileType {
        self.kind
    }

    pub fn is_dir(&self) -> bool {
        self.kind == FileType::Dir
    }

    pub fn is_file(&self) -> bool {
        self.kind == FileType::File
    }

    pub fn is_device(&self) -> bool {
        self.kind == FileType::Device
    }

    pub fn addr(&self) -> u32 {
        self.addr
    }

    pub fn dir(&self) -> Dir {
        self.dir.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn time(&self) -> u64 {
        self.time
    }

    pub fn info(&self) -> FileInfo {
        FileInfo { kind: self.kind, name: self.name(), size: self.size(), time: self.time }
    }
}

#[derive(Debug)]
pub struct FileInfo {
    kind: FileType,
    size: u32,
    time: u64,
    name: String,
}

impl FileInfo {
    pub fn new() -> Self {
        Self { kind: FileType::File, name: String::new(), size: 0, time: 0 }
    }

    pub fn root() -> Self {
        let kind = FileType::Dir;
        let name = String::new();
        let size = Dir::root().size() as u32;
        let time = 0;
        Self { kind, name, size, time }
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn time(&self) -> u64 {
        self.time
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn kind(&self) -> FileType {
        self.kind
    }

    // TODO: Duplicated from dir entry
    pub fn is_dir(&self) -> bool {
        self.kind == FileType::Dir
    }

    pub fn is_file(&self) -> bool {
        self.kind == FileType::File
    }

    pub fn is_device(&self) -> bool {
        self.kind == FileType::Device
    }

    // TODO: Use bincode?
    pub fn as_bytes(&self) -> Vec<u8> {
        debug_assert!(self.name.len() < 256);
        let mut res = Vec::new();
        res.push(self.kind as u8);
        res.extend_from_slice(&self.size.to_be_bytes());
        res.extend_from_slice(&self.time.to_be_bytes());
        res.push(self.name.len() as u8);
        res.extend_from_slice(self.name.as_bytes());
        res
    }
}

use core::convert::TryInto;
use core::convert::From;
impl From<&[u8]> for FileInfo {
    fn from(buf: &[u8]) -> Self {
        let kind = match buf[0] { // TODO: Add FileType::from(u8)
            0 => FileType::Dir,
            1 => FileType::File,
            2 => FileType::Device,
            _ => panic!(),
        };
        let size = u32::from_be_bytes(buf[1..5].try_into().unwrap());
        let time = u64::from_be_bytes(buf[5..13].try_into().unwrap());
        let i = 14 + buf[13] as usize;
        let name = String::from_utf8_lossy(&buf[14..i]).into();
        Self { kind, name, size, time }
    }
}
