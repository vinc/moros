use alloc::vec::Vec;
use core::convert::TryInto;

pub struct Palette {
    pub colors: [(u8, u8, u8, u8); 16]
}

pub fn from_csv(s: &str) -> Result<Palette, ()> {
    let colors: Vec<_> = s.split("\n").filter_map(|line| {
        let line = line.split("#").next().unwrap(); // Remove comments
        let color: Vec<u8> = line.split(",").filter_map(|value| {
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
