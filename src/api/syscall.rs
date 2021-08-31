use crate::syscall;
use crate::sys::syscall::number::*;

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

pub fn open(path: &str, mode: u8) -> u16 {
    let ptr = path.as_ptr() as usize;
    let len = path.len() as usize;
    let res = unsafe { syscall!(OPEN, ptr, len, mode as usize) };
    res as u16
}

pub fn read(fh: u16, buf: &mut [u8]) -> usize {
    let ptr = buf.as_ptr() as usize;
    let len = buf.len() as usize;
    let res = unsafe { syscall!(READ, fh, ptr, len) };
    res as usize
}

pub fn write(fh: u16, buf: &mut [u8]) -> usize {
    let ptr = buf.as_ptr() as usize;
    let len = buf.len() as usize;
    let res = unsafe { syscall!(WRITE, fh, ptr, len) };
    res as usize
}

pub fn close(fh: u16) {
    unsafe { syscall!(CLOSE, fh as usize) };
}

#[test_case]
fn test_open() {
    use crate::sys::fs::{mount_mem, format_mem, File, dismount};
    mount_mem();
    format_mem();
    assert!(File::create("/test").is_some());
    //assert_eq!(open("/test", 0), 4);
    dismount();
}
