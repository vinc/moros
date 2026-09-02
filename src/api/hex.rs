use super::console::Style;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub fn format_hex(buf: &[u8]) -> String {
    format_hex_at(buf, 0)
}

pub fn format_hex_at(buf: &[u8], offset: usize) -> String {
    let null = 0 as char;
    let cyan = Style::color("aqua");
    let gray = Style::color("gray");
    let pink = Style::color("fushia");
    let reset = Style::reset();

    let mut res = String::new();
    for (index, chunk) in buf.chunks(16).enumerate() {
        let addr = offset + index * 16;

        let hex = chunk.chunks(2).map(|pair|
            pair.iter().map(|byte|
                format!("{:02X}", byte)
            ).collect::<Vec<String>>().join("")
        ).collect::<Vec<String>>().join(" ");

        let ascii: String = chunk.iter().map(|byte|
            if *byte >= 32 && *byte <= 126 {
                *byte as char
            } else {
                null
            }
        ).collect();

        let text = ascii.replace(null, &format!("{}.{}", gray, reset));

        res.push_str(&format!(
            "{}{:08X}: {}{:40}{}{}\n", cyan, addr, pink, hex, reset, text
        ));
    }
    res
}
