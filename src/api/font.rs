use alloc::vec::Vec;

#[derive(Clone)]
pub struct Font {
    pub height: u8,
    pub size: u16,
    pub data: Vec<u8>,
}

impl Font {
    // http://www.fifi.org/doc/console-tools-dev/file-formats/psf
    pub fn from_bytes(buf: &[u8]) -> Result<Font, ()> {
        // Header
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

        // Data
        let n = (4 + size * height as u16) as usize;
        if buf.len() < n {
            return Err(());
        }
        let data = buf[4..n].to_vec();

        // TODO: Unicode Table

        Ok(Font { height, size, data })
    }
}

#[test_case]
fn parse_psf_font() {
    let buf = include_bytes!("../../dsk/ini/boot.sh");
    assert!(Font::from_bytes(buf).is_err());

    let buf = include_bytes!("../../dsk/ini/fonts/zap-light-8x16.psf");
    let font = Font::from_bytes(buf).unwrap();
    assert_eq!(font.height, 16);
    assert_eq!(font.size, 256);
    assert_eq!(font.data.len(), 256 * 16);
}
