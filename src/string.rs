use core::{str, slice};

pub struct String {
    buffer: [u8; 256],
    len: usize
}

impl String {
    pub fn new() -> Self {
        Self {
            buffer: [0; 256],
            len: 0
        }
    }

    pub fn push(&mut self, c: u8) {
        let n = self.len;
        self.buffer[n] = c;
        self.len += 1;
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.buffer.as_ptr() as *const u8, self.len) }
    }

    pub fn as_str(&self) -> &str {
        str::from_utf8(self.as_slice()).unwrap()
    }
}
