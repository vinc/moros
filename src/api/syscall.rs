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

pub fn open(path: &str, mode: u8) -> Option<usize> {
    let ptr = path.as_ptr() as usize;
    let len = path.len() as usize;
    let res = unsafe { syscall!(OPEN, ptr, len, mode as usize) } as isize;
    if res.is_negative() {
        None
    } else {
        Some(res as usize)
    }
}

pub fn read(fh: usize, buf: &mut [u8]) -> Option<usize> {
    let ptr = buf.as_ptr() as usize;
    let len = buf.len() as usize;
    let res = unsafe { syscall!(READ, fh, ptr, len) } as isize;
    if res.is_negative() {
        None
    } else {
        Some(res as usize)
    }
}

pub fn write(fh: usize, buf: &mut [u8]) -> Option<usize> {
    let ptr = buf.as_ptr() as usize;
    let len = buf.len() as usize;
    let res = unsafe { syscall!(WRITE, fh, ptr, len) } as isize;
    if res.is_negative() {
        None
    } else {
        Some(res as usize)
    }
}

pub fn close(fh: usize) {
    unsafe { syscall!(CLOSE, fh as usize) };
}

#[test_case]
fn test_open() {
    use crate::sys::fs::{mount_mem, format_mem, File, dismount};
    mount_mem();
    format_mem();
    assert!(File::create("/test1").is_some());
    // FIXME: allocator panic
    // assert_eq!(open("/test1", 0), Some(4));
    // assert_eq!(open("/test2", 0), None);
    dismount();
}
