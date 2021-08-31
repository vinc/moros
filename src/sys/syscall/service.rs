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

pub fn open(path: &str, _mode: u8) -> isize {
    if let Some(file) = sys::fs::File::open(path) {
        if let Ok(fh) = sys::process::create_file_handle(file) {
            return fh as isize;
        }
    }
    -1
}

pub fn read(fh: usize, buf: &mut [u8]) -> isize {
    if let Some(mut file) = sys::process::file_handle(fh) {
        let bytes = file.read(buf);
        sys::process::update_file_handle(fh, file);
        return bytes as isize;
    }
    -1
}

pub fn write(fh: usize, buf: &mut [u8]) -> isize {
    if let Some(mut file) = sys::process::file_handle(fh) {
        if let Ok(bytes) = file.write(buf) {
            sys::process::update_file_handle(fh, file);
            return bytes as isize;
        }
    }
    -1
}

pub fn close(fh: usize) {
    sys::process::delete_file_handle(fh);
}
