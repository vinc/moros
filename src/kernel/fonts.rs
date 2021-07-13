use alloc::vec::Vec;

#[derive(Clone)]
pub struct Font {
    pub height: u8,
    pub size: u16,
    pub data: Vec<u8>,
}

// http://www.fifi.org/doc/console-tools-dev/file-formats/psf
pub fn from_bytes(buf: &[u8]) -> Result<Font, ()> {
    if buf.len() < 4 || buf[0] != 0x36 || buf[1] != 0x04 {
        return Err(());
    }
    let mode = buf[2];
    let height = buf[3];
    let size = match mode {
        0 | 2 => 256,
        1 | 3 => 512,
        _ => return Err(()),
    };
    let n = (4 + size * height as u16) as usize;
    if buf.len() < n {
        return Err(());
    }
    let data = buf[4..n].to_vec();
    Ok(Font { height, size, data })
}
