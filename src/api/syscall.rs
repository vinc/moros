use crate::syscall;
use crate::sys::syscall::number::*;
use crate::sys::fs::FileStat;

pub fn sleep(seconds: f64) {
    unsafe { syscall!(SLEEP, seconds.to_bits()) };
}

pub fn uptime() -> f64 {
    let res = unsafe { syscall!(UPTIME) };
    f64::from_bits(res as u64)
}

pub fn realtime() -> f64 {
    let res = unsafe { syscall!(REALTIME) };
    f64::from_bits(res as u64)
}

pub fn stat(path: &str) -> Option<FileStat> {
    let path_ptr = path.as_ptr() as usize;
    let path_len = path.len() as usize;
    let mut stat = FileStat::new();
    let stat_ptr = &mut stat as *mut FileStat as usize;
    let res = unsafe { syscall!(STAT, path_ptr, path_len, stat_ptr) } as isize;
    if res.is_negative() {
        None
    } else {
        Some(stat)
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
