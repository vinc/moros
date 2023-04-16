use alloc::vec::Vec;
use core::convert::TryInto;

pub struct Palette {
    pub colors: [(u8, u8, u8); 16]
}

impl Palette {
    pub fn default() -> Palette {
        Palette {
            colors: [
                (0x00, 0x00, 0x00), // Black
                (0x00, 0x00, 0x80), // Blue
                (0x00, 0x80, 0x00), // Green
                (0x00, 0x80, 0x80), // Cyan
                (0x80, 0x00, 0x00), // Red
                (0x80, 0x00, 0x80), // Magenta
                (0x80, 0x80, 0x00), // Brown (Dark Yellow)
                (0xC0, 0xC0, 0xC0), // Light Gray
                (0x80, 0x80, 0x80), // Dark Gray (Gray)
                (0x00, 0x00, 0xFF), // Light Blue
                (0x00, 0xFF, 0x00), // Light Green
                (0x00, 0xFF, 0xFF), // Light Cyan
                (0xFF, 0x00, 0x00), // Light Red
                (0xFF, 0x00, 0xFF), // Pink (Light Magenta)
                (0xFF, 0xFF, 0x00), // Yellow (Light Yellow)
                (0xFF, 0xFF, 0xFF), // White
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
        if color.len() == 3 { // RGB values
            Some((color[0], color[1], color[2]))
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

    let s = include_str!("../../../dsk/ini/palettes/gruvbox-dark.csv");
    let palette = from_csv(s).unwrap();
    assert_eq!(palette.colors[0x03].0, 0x68);
    assert_eq!(palette.colors[0x0D].1, 0x86);
}
