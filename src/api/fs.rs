use crate::api::syscall;
use crate::sys;
use crate::sys::fs::OpenFlag;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

pub use crate::sys::fs::{DeviceType, FileInfo};

#[derive(Clone, Copy)]
pub enum IO {
    Read,
    Write,
}

pub trait FileIO {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()>;
    fn write(&mut self, buf: &[u8]) -> Result<usize, ()>;
    fn close(&mut self);
    fn poll(&mut self, event: IO) -> bool;
}

pub fn dirname(pathname: &str) -> &str {
    let pathname = if pathname.len() > 1 {
        pathname.trim_end_matches('/')
    } else {
        pathname
    };
    let i = match pathname.rfind('/') {
        Some(0) => 1,
        Some(i) => i,
        None => return "",
    };
    &pathname[0..i]
}

pub fn filename(pathname: &str) -> &str {
    let pathname = if pathname.len() > 1 {
        pathname.trim_end_matches('/')
    } else {
        pathname
    };
    let n = pathname.len();
    let i = match pathname.rfind('/') {
        Some(i) => i + 1,
        None => 0,
    };
    &pathname[i..n]
}

// Transform "foo.txt" into "/path/to/foo.txt"
pub fn realpath(pathname: &str) -> String {
    if pathname.starts_with('/') {
        pathname.into()
    } else {
        let dirname = sys::process::dir();
        let sep = if dirname.ends_with('/') { "" } else { "/" };
        format!("{}{}{}", dirname, sep, pathname)
    }
}

pub fn is_absolute_path(path: &str) -> bool {
    path.starts_with('/') || path.starts_with('~')
}

pub fn exists(path: &str) -> bool {
    syscall::info(path).is_some()
}

pub fn is_dir(path: &str) -> bool {
    if let Some(info) = syscall::info(path) {
        info.is_dir()
    } else {
        false
    }
}

pub fn is_file(path: &str) -> bool {
    if let Some(info) = syscall::info(path) {
        info.is_file()
    } else {
        false
    }
}

pub fn is_device(path: &str) -> bool {
    if let Some(info) = syscall::info(path) {
        info.is_device()
    } else {
        false
    }
}

pub fn delete(path: &str) -> Result<(), ()> {
    syscall::delete(path)
}

pub fn open_file(path: &str) -> Option<usize> {
    let flags = 0;
    syscall::open(path, flags)
}

pub fn append_file(path: &str) -> Option<usize> {
    let flags = OpenFlag::Append as usize;
    syscall::open(path, flags)
}

pub fn create_file(path: &str) -> Option<usize> {
    let flags = OpenFlag::Create as usize;
    syscall::open(path, flags)
}

pub fn open_dir(path: &str) -> Option<usize> {
    let flags = OpenFlag::Dir as usize;
    syscall::open(path, flags)
}

pub fn create_dir(path: &str) -> Option<usize> {
    let flags = OpenFlag::Create as usize | OpenFlag::Dir as usize;
    syscall::open(path, flags)
}

pub fn open_device(path: &str) -> Option<usize> {
    let flags = OpenFlag::Device as usize;
    syscall::open(path, flags)
}

pub fn create_device(path: &str, name: &str) -> Option<usize> {
    if let Ok(buf) = device_buffer(name) {
        let flags = OpenFlag::Create as usize | OpenFlag::Device as usize;
        if let Some(handle) = syscall::open(path, flags) {
            syscall::write(handle, &buf);
            return Some(handle);
        }
    }
    None
}

fn device_buffer(name: &str) -> Result<Vec<u8>, ()> {
    let arg = if name.starts_with("ata-") { "ata" } else { name };
    let dev = device_type(arg)?;
    let mut buf = dev.buf();
    if name.starts_with("ata-") {
        match name {
            "ata-0-0" => { buf[1] = 0; buf[2] = 0 },
            "ata-0-1" => { buf[1] = 0; buf[2] = 1 },
            "ata-1-0" => { buf[1] = 1; buf[2] = 0 },
            "ata-1-1" => { buf[1] = 1; buf[2] = 1 },
            _ => return Err(()),
        }
    }
    Ok(buf)
}

// TODO: Move this to sys::fs::device
fn device_type(name: &str) -> Result<DeviceType, ()> {
    match name {
        "null"        => Ok(DeviceType::Null),
        "file"        => Ok(DeviceType::File),
        "console"     => Ok(DeviceType::Console),
        "random"      => Ok(DeviceType::Random),
        "clk-boot"    => Ok(DeviceType::BootTime),
        "clk-epoch"   => Ok(DeviceType::EpochTime),
        "clk-rtc"     => Ok(DeviceType::RTC),
        "tcp"         => Ok(DeviceType::TcpSocket),
        "udp"         => Ok(DeviceType::UdpSocket),
        "vga-buffer"  => Ok(DeviceType::VgaBuffer),
        "vga-font"    => Ok(DeviceType::VgaFont),
        "vga-mode"    => Ok(DeviceType::VgaMode),
        "vga-palette" => Ok(DeviceType::VgaPalette),
        "speaker"     => Ok(DeviceType::Speaker),
        "ata"         => Ok(DeviceType::Drive),
        _             => Err(()),
    }
}

pub fn read(path: &str, buf: &mut [u8]) -> Result<usize, ()> {
    if let Some(info) = syscall::info(path) {
        let res = if info.is_device() {
            open_device(path)
        } else {
            open_file(path)
        };
        if let Some(handle) = res {
            if let Some(bytes) = syscall::read(handle, buf) {
                syscall::close(handle);
                return Ok(bytes);
            }
        }
    }
    Err(())
}

pub fn read_to_string(path: &str) -> Result<String, ()> {
    let buf = read_to_bytes(path)?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

pub fn read_to_bytes(path: &str) -> Result<Vec<u8>, ()> {
    if let Some(info) = syscall::info(path) {
        let res = if info.is_device() {
            open_device(path)
        } else if info.is_dir() {
            open_dir(path)
        } else {
            open_file(path)
        };
        if let Some(handle) = res {
            let n = info.size() as usize;
            let mut buf = vec![0; n];
            if let Some(bytes) = syscall::read(handle, &mut buf) {
                buf.resize(bytes, 0);
                syscall::close(handle);
                return Ok(buf);
            }
        }
    }
    Err(())
}

pub fn write(path: &str, buf: &[u8]) -> Result<usize, ()> {
    let res = if is_device(path) {
        open_device(path)
    } else {
        create_file(path)
    };
    if let Some(handle) = res {
        if let Some(bytes) = syscall::write(handle, buf) {
            syscall::close(handle);
            return Ok(bytes);
        }
    }
    Err(())
}

pub fn reopen(path: &str, handle: usize, append: bool) -> Result<usize, ()> {
    let res = if let Some(info) = syscall::info(path) {
        if info.is_device() {
            open_device(path)
        } else if append {
            append_file(path)
        } else {
            open_file(path)
        }
    } else {
        create_file(path)
    };
    if let Some(old_handle) = res {
        syscall::dup(old_handle, handle);
        syscall::close(old_handle);
        return Ok(handle);
    }
    Err(())
}

pub fn read_dir(path: &str) -> Result<Vec<FileInfo>, ()> {
    if let Some(info) = syscall::info(path) {
        if info.is_dir() {
            if let Ok(buf) = read_to_bytes(path) {
                let mut res = Vec::new();
                let mut i = 0;
                let n = buf.len();
                while i < n {
                    let j = i + 14 + buf[i + 13] as usize;
                    if j > n {
                        break;
                    }
                    let info = FileInfo::from(&buf[i..j]);
                    res.push(info);
                    i = j;
                }
                return Ok(res);
            }
        }
    }
    Err(())
}

#[test_case]
fn test_filename() {
    assert_eq!(filename("/path/to/file.txt"), "file.txt");
    assert_eq!(filename("/file.txt"), "file.txt");
    assert_eq!(filename("file.txt"), "file.txt");
    assert_eq!(filename("/path/to/"), "to");
    assert_eq!(filename("/path/to"), "to");
    assert_eq!(filename("path/to"), "to");
    assert_eq!(filename("/"), "");
    assert_eq!(filename(""), "");
}

#[test_case]
fn test_dirname() {
    assert_eq!(dirname("/path/to/file.txt"), "/path/to");
    assert_eq!(dirname("/file.txt"), "/");
    assert_eq!(dirname("file.txt"), "");
    assert_eq!(dirname("/path/to/"), "/path");
    assert_eq!(dirname("/path/to"), "/path");
    assert_eq!(dirname("path/to"), "path");
    assert_eq!(dirname("/"), "/");
    assert_eq!(dirname(""), "");
}

#[test_case]
fn test_is_absolute_path() {
    assert_eq!(is_absolute_path("/path/to/binary"), true);
    assert_eq!(is_absolute_path("~/path/to/binary"), true);
    assert_eq!(is_absolute_path("path/to/binary"), false);
    assert_eq!(is_absolute_path("binary"), false);
}

#[test_case]
fn test_fs() {
    use crate::sys::fs::{dismount, format_mem, mount_mem};
    mount_mem();
    format_mem();

    assert_eq!(open_file("/test"), None);

    // Write file
    let input = "Hello, world!".as_bytes();
    assert_eq!(write("/test", &input), Ok(input.len()));

    // Read file
    assert_eq!(read_to_bytes("/test"), Ok(input.to_vec()));

    dismount();
}
