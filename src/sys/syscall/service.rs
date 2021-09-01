use crate::sys;
use crate::sys::fs::FileStat;

pub fn sleep(seconds: f64) {
    unsafe { asm!("sti") }; // Restore interrupts
    sys::time::sleep(seconds);
    unsafe { asm!("cli") }; // Disable interrupts
}

pub fn uptime() -> f64 {
    sys::clock::uptime()
}

pub fn realtime() -> f64 {
    sys::clock::realtime()
}

pub fn stat(path: &str, stat: &mut FileStat) -> isize {
    if let Some(file) = sys::fs::File::open(path) {
        *stat = file.stat();
        0
    } else {
        -1
    }
}

pub fn open(path: &str, flags: usize) -> isize {
    if let Some(file) = sys::fs::open_file(path, flags) {
        if let Ok(handle) = sys::process::create_file_handle(file) {
            return handle as isize;
        }
    }
    -1
}

pub fn read(handle: usize, buf: &mut [u8]) -> isize {
    if let Some(mut file) = sys::process::file_handle(handle) {
        let bytes = file.read(buf);
        sys::process::update_file_handle(handle, file);
        return bytes as isize;
    }
    -1
}

pub fn write(handle: usize, buf: &mut [u8]) -> isize {
    if let Some(mut file) = sys::process::file_handle(handle) {
        if let Ok(bytes) = file.write(buf) {
            sys::process::update_file_handle(handle, file);
            return bytes as isize;
        }
    }
    -1
}

pub fn close(handle: usize) {
    sys::process::delete_file_handle(handle);
}
