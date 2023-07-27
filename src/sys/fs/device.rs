use super::{dirname, filename, realpath, FileIO, IO};
use super::dir::Dir;
use super::file::File;
use super::block::LinkedBlock;

use crate::sys::cmos::RTC;
use crate::sys::console::Console;
use crate::sys::random::Random;
use crate::sys::clock::{Uptime, Realtime};
use crate::sys::net::socket::tcp::TcpSocket;
use crate::sys::net::socket::udp::UdpSocket;

use alloc::vec;
use alloc::vec::Vec;

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum DeviceType {
    Null      = 0,
    File      = 1,
    Console   = 2,
    Random    = 3,
    Uptime    = 4,
    Realtime  = 5,
    RTC       = 6,
    TcpSocket = 7,
    UdpSocket = 8,
}

// Used when creating a device
impl DeviceType {
    pub fn buf(self) -> Vec<u8> {
        let len = match self {
            DeviceType::RTC       => RTC::size(),
            DeviceType::Uptime    => Uptime::size(),
            DeviceType::Realtime  => Realtime::size(),
            DeviceType::Console   => Console::size(),
            _                     => 1,
        };
        let mut res = vec![0; len];
        res[0] = self as u8;
        res
    }
}

#[derive(Debug, Clone)]
pub enum Device {
    Null,
    File(File),
    Console(Console),
    Random(Random),
    Uptime(Uptime),
    Realtime(Realtime),
    RTC(RTC),
    TcpSocket(TcpSocket),
    UdpSocket(UdpSocket),
}

impl From<u8> for Device {
    fn from(i: u8) -> Self {
        match i {
            i if i == DeviceType::Null as u8 => Device::Null,
            i if i == DeviceType::File as u8 => Device::File(File::new()),
            i if i == DeviceType::Console as u8 => Device::Console(Console::new()),
            i if i == DeviceType::Random as u8 => Device::Random(Random::new()),
            i if i == DeviceType::Uptime as u8 => Device::Uptime(Uptime::new()),
            i if i == DeviceType::Realtime as u8 => Device::Realtime(Realtime::new()),
            i if i == DeviceType::RTC as u8 => Device::RTC(RTC::new()),
            i if i == DeviceType::TcpSocket as u8 => Device::TcpSocket(TcpSocket::new()),
            i if i == DeviceType::UdpSocket as u8 => Device::UdpSocket(UdpSocket::new()),
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

    // TODO: Add size()
}

impl FileIO for Device {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        match self {
            Device::Null          => Err(()),
            Device::File(io)      => io.read(buf),
            Device::Console(io)   => io.read(buf),
            Device::Random(io)    => io.read(buf),
            Device::Uptime(io)    => io.read(buf),
            Device::Realtime(io)  => io.read(buf),
            Device::RTC(io)       => io.read(buf),
            Device::TcpSocket(io) => io.read(buf),
            Device::UdpSocket(io) => io.read(buf),
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        match self {
            Device::Null          => Ok(0),
            Device::File(io)      => io.write(buf),
            Device::Console(io)   => io.write(buf),
            Device::Random(io)    => io.write(buf),
            Device::Uptime(io)    => io.write(buf),
            Device::Realtime(io)  => io.write(buf),
            Device::RTC(io)       => io.write(buf),
            Device::TcpSocket(io) => io.write(buf),
            Device::UdpSocket(io) => io.write(buf),
        }
    }

    fn close(&mut self) {
        match self {
            Device::Null          => {},
            Device::File(io)      => io.close(),
            Device::Console(io)   => io.close(),
            Device::Random(io)    => io.close(),
            Device::Uptime(io)    => io.close(),
            Device::Realtime(io)  => io.close(),
            Device::RTC(io)       => io.close(),
            Device::TcpSocket(io) => io.close(),
            Device::UdpSocket(io) => io.close(),
        }
    }

    fn poll(&mut self, event: IO) -> bool {
        match self {
            Device::Null          => false,
            Device::File(io)      => io.poll(event),
            Device::Console(io)   => io.poll(event),
            Device::Random(io)    => io.poll(event),
            Device::Uptime(io)    => io.poll(event),
            Device::Realtime(io)  => io.poll(event),
            Device::RTC(io)       => io.poll(event),
            Device::TcpSocket(io) => io.poll(event),
            Device::UdpSocket(io) => io.poll(event),
        }
    }
}
