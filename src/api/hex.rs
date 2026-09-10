use super::console::Style;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

pub struct HexDump<'a> {
    buf: &'a [u8],
    offset: usize,
}

pub fn format_hex(buf: &[u8]) -> HexDump<'_> {
    format_hex_at(buf, 0)
}

pub fn format_hex_at(buf: &[u8], offset: usize) -> HexDump<'_> {
    HexDump { buf, offset }
}

impl fmt::Display for HexDump<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let null = 0 as char;
        let cyan = Style::color("aqua");
        let gray = Style::color("gray");
        let pink = Style::color("fushia");
        let reset = Style::reset();

        for (index, chunk) in self.buf.chunks(16).enumerate() {
            let addr = self.offset + index * 16;

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
            write!(
                f, "{}{:08X}: {}{:40}{}{}\n",
                cyan, addr, pink, hex, reset, text
            )?;
        }

        Ok(())
    }
}
