use crate::syscall;
use crate::sys::syscall::number::*;
use crate::sys::fs::FileInfo;

pub fn exit(code: usize) -> usize {
    unsafe { syscall!(EXIT, code as u64) }
}

pub fn sleep(seconds: f64) {
    unsafe { syscall!(SLEEP, seconds.to_bits()) };
}

pub fn delete(path: &str) -> Result<(), ()> {
    let path_ptr = path.as_ptr() as usize;
    let path_len = path.len() as usize;
    let res = unsafe { syscall!(DELETE, path_ptr, path_len) } as isize;
    if res.is_negative() {
        Err(())
    } else {
        Ok(())
    }
}

pub fn info(path: &str) -> Option<FileInfo> {
    let path_ptr = path.as_ptr() as usize;
    let path_len = path.len() as usize;
    let mut info = FileInfo::new();
    let stat_ptr = &mut info as *mut FileInfo as usize;
    let res = unsafe { syscall!(INFO, path_ptr, path_len, stat_ptr) } as isize;
    if res.is_negative() {
        None
    } else {
        Some(info)
    }
}

pub fn open(path: &str, flags: usize) -> Option<usize> {
    let ptr = path.as_ptr() as usize;
    let len = path.len() as usize;
    let res = unsafe { syscall!(OPEN, ptr, len, flags) } as isize;
    if res.is_negative() {
        None
    } else {
        Some(res as usize)
    }
}

pub fn dup(old_handle: usize, new_handle: usize) -> Option<usize> {
    let res = unsafe { syscall!(DUP, old_handle, new_handle) } as isize;
    if res.is_negative() {
        None
    } else {
        Some(res as usize)
    }
}

pub fn read(handle: usize, buf: &mut [u8]) -> Option<usize> {
    let ptr = buf.as_ptr() as usize;
    let len = buf.len() as usize;
    let res = unsafe { syscall!(READ, handle, ptr, len) } as isize;
    if res.is_negative() {
        None
    } else {
        Some(res as usize)
    }
}

pub fn write(handle: usize, buf: &[u8]) -> Option<usize> {
    let ptr = buf.as_ptr() as usize;
    let len = buf.len() as usize;
    let res = unsafe { syscall!(WRITE, handle, ptr, len) } as isize;
    if res.is_negative() {
        None
    } else {
        Some(res as usize)
    }
}

pub fn close(handle: usize) {
    unsafe { syscall!(CLOSE, handle as usize) };
}

pub fn spawn(path: &str) {
    let ptr = path.as_ptr() as usize;
    let len = path.len() as usize;
    unsafe { syscall!(SPAWN, ptr, len) };
}

pub fn reboot() {
    unsafe { syscall!(STOP, 0xcafe) };
}

pub fn halt() {
    unsafe { syscall!(STOP, 0xdead) };
}

#[test_case]
fn test_file() {
    use crate::sys::fs::{mount_mem, format_mem, dismount, OpenFlag};
    use alloc::vec;
    mount_mem();
    format_mem();

    let flags = 0;
    assert_eq!(open("/test", flags), None);

    // Write file
    let flags = OpenFlag::Create as usize;
    assert_eq!(open("/test", flags), Some(4));
    let input = "Hello, world!".as_bytes();
    assert_eq!(write(4, &input), Some(input.len()));

    // Read file
    let flags = 0;
    assert_eq!(open("/test", flags), Some(5));
    let mut output = vec![0; input.len()];
    assert_eq!(read(5, &mut output), Some(input.len()));
    assert_eq!(output, input);

    close(4);
    close(5);

    assert_eq!(open("/test", flags), Some(4));

    close(4);

    //assert!(write(1, b"Hello, World\n").is_some());

    dismount();
}
