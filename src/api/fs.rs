use crate::sys;
use crate::sys::fs::OpenFlag;
use crate::api::syscall;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;

pub use crate::sys::fs::{FileInfo, DeviceType};

#[derive(Clone, Copy)]
pub enum IO { Read, Write }

pub trait FileIO {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()>;
    fn write(&mut self, buf: &[u8]) -> Result<usize, ()>;
    fn close(&mut self);
    fn poll(&mut self, event: IO) -> bool;
}

pub fn dirname(pathname: &str) -> &str {
    let n = pathname.len();
    let i = match pathname.rfind('/') {
        Some(0) => 1,
        Some(i) => i,
        None => n,
    };
    &pathname[0..i]
}

pub fn filename(pathname: &str) -> &str {
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

pub fn create_device(path: &str, kind: DeviceType) -> Option<usize> {
    let flags = OpenFlag::Create as usize | OpenFlag::Device as usize;
    if let Some(handle) = syscall::open(path, flags) {
        syscall::write(handle, &kind.buf());
        return Some(handle);
    }
    None
}

pub fn read(path: &str, buf: &mut [u8]) -> Result<usize, ()> {
    if let Some(info) = syscall::info(path) {
        let res = if info.is_device() { open_device(path) } else { open_file(path) };
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
        let f = if info.is_device() {
            open_device(path)
        } else if info.is_dir() {
            open_dir(path)
        } else {
            open_file(path)
        };
        if let Some(handle) = f {
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
    if let Some(handle) = create_file(path) {
        if let Some(bytes) = syscall::write(handle, buf) {
            syscall::close(handle);
            return Ok(bytes);
        }
    }
    Err(())
}

pub fn append(path: &str, buf: &[u8]) -> Result<usize, ()> {
    let res = if let Some(info) = syscall::info(path) {
        if info.is_device() {
            open_device(path)
        } else {
            append_file(path)
        }
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

pub fn reopen(path: &str, handle: usize, append_mode: bool) -> Result<usize, ()> {
    let res = if let Some(info) = syscall::info(path) {
        if info.is_device() {
            open_device(path)
        } else if append_mode {
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
fn test_file() {
    use crate::sys::fs::{mount_mem, format_mem, dismount};
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
