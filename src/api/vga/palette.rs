use alloc::vec::Vec;
use core::convert::TryInto;

pub struct Palette {
    pub colors: [(u8, u8, u8, u8); 16]
}

impl Palette {
    pub fn default() -> Palette {
        Palette {
            colors: [
                (0x00, 0x00, 0x00, 0x00), // Black
                (0x01, 0x00, 0x00, 0x80), // Blue
                (0x02, 0x00, 0x80, 0x00), // Green
                (0x03, 0x00, 0x80, 0x80), // Cyan
                (0x04, 0x80, 0x00, 0x00), // Red
                (0x05, 0x80, 0x00, 0x80), // Magenta
                (0x06, 0x80, 0x80, 0x00), // Brown (Dark Yellow)
                (0x07, 0xC0, 0xC0, 0xC0), // Light Gray
                (0x08, 0x80, 0x80, 0x80), // Dark Gray (Gray)
                (0x09, 0x00, 0x00, 0xFF), // Light Blue
                (0x0A, 0x00, 0xFF, 0x00), // Light Green
                (0x0B, 0x00, 0xFF, 0xFF), // Light Cyan
                (0x0C, 0xFF, 0x00, 0x00), // Light Red
                (0x0D, 0xFF, 0x00, 0xFF), // Pink (Light Magenta)
                (0x0E, 0xFF, 0xFF, 0x00), // Yellow (Light Yellow)
                (0x0F, 0xFF, 0xFF, 0xFF), // White
            ]
        }
    }
}

pub fn from_csv(s: &str) -> Result<Palette, ()> {
    let colors: Vec<_> = s.split('\n').filter_map(|line| {
        let line = line.split('#').next().unwrap(); // Remove comments
        let color: Vec<u8> = line.split(',').filter_map(|value| {
            let radix = if value.contains("0x") { 16 } else { 10 };
            let value = value.trim().trim_start_matches("0x");
            u8::from_str_radix(value, radix).ok()
        }).collect();
        if color.len() == 4 { // Color index + rgb values
            Some((color[0], color[1], color[2], color[3]))
        } else {
            None
        }
    }).collect();
    if let Ok(colors) = colors.try_into() { // Array of 16 colors
        Ok(Palette { colors })
    } else {
        Err(())
    }
}

#[test_case]
fn parse_palette_csv() {
    assert!(from_csv("").is_err());
    assert!(from_csv("0,0,0,0").is_err());

    let s = include_str!("../../../dsk/ini/palette.csv");
    let palette = from_csv(s).unwrap();
    assert_eq!(palette.colors[0x03].0, 0x03);
    assert_eq!(palette.colors[0x0D].2, 0x86);
}
