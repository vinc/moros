use crate::api::syscall;
use crate::sys::fs::OpenFlag;
use crate::sys;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

pub fn open(path: &str) -> Option<usize> {
    let flags = 0;
    syscall::open(path, flags)
}

pub fn create(path: &str) -> Option<usize> {
    let flags = OpenFlag::Create as usize;
    syscall::open(path, flags)
}

pub fn canonicalize(path: &str) -> Result<String, ()> {
    match sys::process::env("HOME") {
        Some(home) => {
            if path.starts_with('~') {
                Ok(path.replace("~", &home))
            } else {
                Ok(path.to_string())
            }
        },
        None => {
            Ok(path.to_string())
        }
    }
}

pub fn read_to_string(path: &str) -> Result<String, ()> {
    let buf = read(path)?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

pub fn read(path: &str) -> Result<Vec<u8>, ()> {
    let path = match canonicalize(path) {
        Ok(path) => path,
        Err(_) => return Err(()),
    };
    if let Some(stat) = syscall::stat(&path) {
        if let Some(handle) = open(&path) {
            let mut buf = vec![0; stat.size() as usize];
            if let Some(bytes) = syscall::read(handle, &mut buf) {
                buf.resize(bytes, 0);
                syscall::close(handle);
                return Ok(buf)
            }
        }
    }
    Err(())
}

pub fn write(path: &str, buf: &[u8]) -> Result<usize, ()> {
    let path = match canonicalize(path) {
        Ok(path) => path,
        Err(_) => return Err(()),
    };
    if let Some(handle) = create(&path) {
        if let Some(bytes) = syscall::write(handle, buf) {
            syscall::close(handle);
            return Ok(bytes)
        }
    }
    Err(())
}

#[test_case]
fn test_file() {
    use crate::sys::fs::{mount_mem, format_mem, dismount};
    mount_mem();
    format_mem();

    assert_eq!(open("/test"), None);

    // Write file
    let input = "Hello, world!".as_bytes();
    assert_eq!(write("/test", &input), Ok(input.len()));

    // Read file
    assert_eq!(read("/test"), Ok(input.to_vec()));

    dismount();
}
