use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

// TODO: add `--skip` and `--length` params
pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 2 {
        help();
        return Err(ExitCode::UsageError);
    }
    if args[1] == "-h" || args[1] == "--help" {
        help();
        return Ok(());
    }
    let pathname = args[1];
    if let Ok(buf) = fs::read_to_bytes(pathname) {
        // TODO: read chunks
        print_hex(&buf);
        Ok(())
    } else {
        error!("Could not read file '{}'", pathname);
        Err(ExitCode::Failure)
    }
}

// TODO: move this to api::hex::print_hex
pub fn print_hex(buf: &[u8]) {
    print_hex_at(buf, 0)
}

pub fn print_hex_at(buf: &[u8], offset: usize) {
    let null = 0 as char;
    let cyan = Style::color("LightCyan");
    let gray = Style::color("DarkGray");
    let pink = Style::color("Pink");
    let reset = Style::reset();

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

        println!("{}{:08X}: {}{:40}{}{}", cyan, addr, pink, hex, reset, text);
    }
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} hex {}<file>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
}
