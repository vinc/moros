use crate::sys;

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

pub fn open(path: &str, _mode: u8) -> u16 {
    if let Some(file) = sys::fs::File::open(path) {
        if let Ok(fh) = sys::process::create_file_handle(file) {
            return fh as u16;
        }
    }
    0
}

pub fn read(fh: u16, buf: &mut [u8]) -> usize {
    if let Some(mut file) = sys::process::file_handle(fh as usize) {
        let bytes = file.read(buf);
        sys::process::update_file_handle(fh as usize, file);
        return bytes;
    }
    0
}

pub fn write(fh: u16, buf: &mut [u8]) -> usize {
    if let Some(mut file) = sys::process::file_handle(fh as usize) {
        if let Ok(bytes) = file.write(buf) {
            sys::process::update_file_handle(fh as usize, file);
            return bytes;
        }
    }
    0
}

pub fn close(fh: u16) {
    sys::process::delete_file_handle(fh as usize);
}
