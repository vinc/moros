use alloc::vec::Vec;
use core::convert::TryInto;

pub struct Palette {
    pub colors: [(u8, u8, u8, u8); 16]
}

pub fn from_csv(s: &str) -> Result<Palette, ()> {
    let mut colors = Vec::with_capacity(16);
    for row in s.split("\n") {
        let row = row.split("#").next().unwrap();
        let color: Vec<u8> = row.split(",").filter_map(|value| {
            let radix = if value.contains("0x") { 16 } else { 10 };
            let value = value.trim().trim_start_matches("0x");
            u8::from_str_radix(value, radix).ok()
        }).collect();
        if color.len() == 4 {
            colors.push((color[0], color[1], color[2], color[3]));
        }
    }
    if let Ok(colors) = colors.try_into() {
        Ok(Palette { colors })
    } else {
        Err(())
    }
}
