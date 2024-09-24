use alloc::string::String;
use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts;

lazy_static! {
    static ref LOG: Mutex<LogBuffer> = Mutex::new(LogBuffer::new());
}

const LOG_SIZE: usize = 10 << 10; // 10 KB

struct LogBuffer {
    buf: [u8; LOG_SIZE],
    len: usize,
}

impl LogBuffer {
    const fn new() -> Self {
        Self {
            buf: [0; LOG_SIZE],
            len: 0,
        }
    }

    fn buf(&self) -> &[u8] {
        let n = self.len;
        &self.buf[0..n]
    }
}

impl core::fmt::Write for LogBuffer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.len + s.len() > LOG_SIZE {
            self.len = 0;
            self.buf.fill(0);
        }

        let bytes = s.as_bytes();
        let i = self.len;
        let n = i + bytes.len();

        self.buf[i..n].copy_from_slice(bytes);
        self.len += bytes.len();

        Ok(())
    }
}

#[doc(hidden)]
pub fn write_fmt(args: fmt::Arguments) {
    interrupts::without_interrupts(||
        LOG.lock().write_fmt(args).expect("Could not write log")
    )
}

pub fn read() -> String {
    let log = LOG.lock();
    let buf = String::from_utf8_lossy(log.buf());
    buf.into_owned()
}
