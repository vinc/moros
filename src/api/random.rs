use crate::api::syscall;
use crate::api::fs;

pub fn get_u64() -> u64 {
    let mut buf = [0; 8];
    if let Some(handle) = fs::open_device("/dev/random") {
        if syscall::read(handle, &mut buf).is_some() {
            syscall::close(handle);
            return u64::from_be_bytes(buf);
        }
    }
    0
}

pub fn get_u16() -> u16 {
    let mut buf = [0; 2];
    if let Some(handle) = fs::open_device("/dev/random") {
        if syscall::read(handle, &mut buf).is_some() {
            syscall::close(handle);
            return u16::from_be_bytes(buf);
        }
    }
    0
}
