use super::block::LinkedBlock;
use super::dir::Dir;
use super::file::File;
use super::{dirname, filename, realpath, FileIO, IO};

use crate::sys::ata::Drive;
use crate::sys::clock::{Realtime, Uptime};
use crate::sys::cmos::RTC;
use crate::sys::console::Console;
use crate::sys::net::socket::tcp::TcpSocket;
use crate::sys::net::socket::udp::UdpSocket;
use crate::sys::random::Random;

use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::convert::TryInto;

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
    Drive     = 9,
}

impl TryFrom<&[u8]> for DeviceType {
    type Error = ();

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        match buf.first().ok_or(())? {
            0 => Ok(DeviceType::Null),
            1 => Ok(DeviceType::File),
            2 => Ok(DeviceType::Console),
            3 => Ok(DeviceType::Random),
            4 => Ok(DeviceType::Uptime),
            5 => Ok(DeviceType::Realtime),
            6 => Ok(DeviceType::RTC),
            7 => Ok(DeviceType::TcpSocket),
            8 => Ok(DeviceType::UdpSocket),
            9 => Ok(DeviceType::Drive),
            _ => Err(()),
        }
    }
}

impl DeviceType {
    // Return a buffer for the file representing the device in the filesystem.
    // The first byte is the device type. The remaining bytes can be used to
    // store specific device informations.
    pub fn buf(self) -> Vec<u8> {
        let len = match self {
            DeviceType::RTC       => RTC::size(),
            DeviceType::Uptime    => Uptime::size(),
            DeviceType::Realtime  => Realtime::size(),
            DeviceType::Console   => Console::size(),
            DeviceType::TcpSocket => TcpSocket::size(),
            DeviceType::UdpSocket => UdpSocket::size(),
            DeviceType::Drive     => Drive::size(),
            _                     => 1,
        };
        let mut res = vec![0; len];
        res[0] = self as u8; // Device type
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
    Drive(Drive),
}

impl TryFrom<&[u8]> for Device {
    type Error = ();

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        match buf.try_into()? {
            DeviceType::Null      => Ok(Device::Null),
            DeviceType::File      => Ok(Device::File(File::new())),
            DeviceType::Console   => Ok(Device::Console(Console::new())),
            DeviceType::Random    => Ok(Device::Random(Random::new())),
            DeviceType::Uptime    => Ok(Device::Uptime(Uptime::new())),
            DeviceType::Realtime  => Ok(Device::Realtime(Realtime::new())),
            DeviceType::RTC       => Ok(Device::RTC(RTC::new())),
            DeviceType::TcpSocket => Ok(Device::TcpSocket(TcpSocket::new())),
            DeviceType::UdpSocket => Ok(Device::UdpSocket(UdpSocket::new())),
            DeviceType::Drive if buf.len() > 2 => {
                let bus = buf[1];
                let dsk = buf[2];
                if let Some(drive) = Drive::open(bus, dsk) {
                    Ok(Device::Drive(drive))
                } else {
                    Err(())
                }
            }
            _ => Err(()),
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
                return Some(Device::File(dir_entry.into()));
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
                    return data.try_into().ok();
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
            Device::Drive(io)     => io.read(buf),
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
            Device::Drive(io)     => io.write(buf),
        }
    }

    fn close(&mut self) {
        match self {
            Device::Null          => {}
            Device::File(io)      => io.close(),
            Device::Console(io)   => io.close(),
            Device::Random(io)    => io.close(),
            Device::Uptime(io)    => io.close(),
            Device::Realtime(io)  => io.close(),
            Device::RTC(io)       => io.close(),
            Device::TcpSocket(io) => io.close(),
            Device::UdpSocket(io) => io.close(),
            Device::Drive(io)     => io.close(),
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
            Device::Drive(io)     => io.poll(event),
        }
    }
}
