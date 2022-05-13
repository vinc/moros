use super::{dirname, filename, realpath, FileIO};
use super::dir::Dir;
use super::file::File;
use super::block::LinkedBlock;

use alloc::vec;
use alloc::vec::Vec;
use crate::sys::console::Console;
use crate::sys::random::Random;

#[derive(PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum DeviceType {
    File = 0,
    Console = 1,
    Random = 2,
    Null = 3,
}

impl DeviceType {
    pub fn buf(self) -> Vec<u8> {
        if self == DeviceType::Console {
            vec![self as u8, 0, 0, 0] // 4 bytes
        } else {
            vec![self as u8] // 1 byte
        }
    }
}

#[derive(Debug, Clone)]
pub enum Device {
    File(File),
    Console(Console),
    Random(Random),
    Null,
}

impl From<u8> for Device {
    fn from(i: u8) -> Self {
        match i {
            i if i == DeviceType::File as u8 => Device::File(File::new()),
            i if i == DeviceType::Console as u8 => Device::Console(Console::new()),
            i if i == DeviceType::Random as u8 => Device::Random(Random::new()),
            i if i == DeviceType::Null as u8 => Device::Null,
            _ => unimplemented!(),
        }
    }
}

impl Device {
    pub fn create(pathname: &str) -> Option<Self> {
        let pathname = realpath(pathname);
        let dirname = dirname(&pathname);
        let filename = filename(&pathname);
        if let Some(mut dir) = Dir::open(dirname) {
            if let Some(dir_entry) = dir.create_device(filename) {
                return Some(Device::File(dir_entry.into()))
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
                if dir_entry.is_device() {
                    let block = LinkedBlock::read(dir_entry.addr());
                    let data = block.data();
                    return Some(data[0].into());
                }
            }
        }
        None
    }
}

impl FileIO for Device {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        match self {
            Device::File(io) => io.read(buf),
            Device::Console(io) => io.read(buf),
            Device::Random(io) => io.read(buf),
            Device::Null => Err(()),
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        match self {
            Device::File(io) => io.write(buf),
            Device::Console(io) => io.write(buf),
            Device::Random(io) => io.write(buf),
            Device::Null => Ok(0),
        }
    }
}
