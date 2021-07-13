use lazy_static::lazy_static;

// https://www.zap.org.au/projects/console-fonts-zap/src/zap-light16.psf
// https://www.zap.org.au/projects/console-fonts-zap/src/zap-vga16.psf
// https://www.zap.org.au/projects/console-fonts-distributed/psftx-debian-10.5/Lat15-Terminus16.psf

pub fn find(name: &str) -> Option<Font> {
    match name {
        "lat15-terminus-8x16" => Some(*LAT15_TERMINUS_8X16),
        "zap-light-8x16"      => Some(*ZAP_LIGHT_8X16),
        "zap-vga-8x16"        => Some(*ZAP_VGA_8X16),
        _                     => None,
    }
}

lazy_static! {
    static ref LAT15_TERMINUS_8X16: Font = from_bytes(include_bytes!("lat15-terminus-8x16.psf")).unwrap();
    static ref ZAP_LIGHT_8X16: Font = from_bytes(include_bytes!("zap-light-8x16.psf")).unwrap();
    static ref ZAP_VGA_8X16: Font = from_bytes(include_bytes!("zap-vga-8x16.psf")).unwrap();
}

// http://www.fifi.org/doc/console-tools-dev/file-formats/psf

#[derive(Clone, Copy)]
pub struct Font {
    pub height: u8,
    pub size: u16,
    pub data: &'static [u8],
}

pub fn from_bytes(buf: &'static [u8]) -> Result<Font, ()> {
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
    let data = &buf[4..n];
    Ok(Font { height, size, data })
}
