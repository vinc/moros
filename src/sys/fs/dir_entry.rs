use super::FileType;
use super::dir::Dir;
use alloc::string::String;

#[derive(Clone)]
pub struct DirEntry {
    dir: Dir,
    kind: FileType,
    addr: u32,
    size: u32,
    time: u64,
    name: String,
}

impl DirEntry {
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

    pub fn is_dir(&self) -> bool {
        self.kind == FileType::Dir
    }

    pub fn is_file(&self) -> bool {
        self.kind == FileType::File
    }

    pub fn addr(&self) -> u32 {
        self.addr
    }

    pub fn dir(&self) -> Dir {
        self.dir
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
}
