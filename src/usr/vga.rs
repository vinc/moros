use crate::{api, sys, usr};
use crate::api::vga::palette;
use crate::api::fs;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 1 {
        eprintln!("Usage: vga <command>");
        return usr::shell::ExitCode::CommandError;
    }
    match args[1] {
        "set" => {
            if args.len() == 4 && args[2] == "font" {
                if let Ok(buf) = fs::read_to_bytes(args[3]) {
                    if let Ok(font) = api::font::from_bytes(&buf) {
                        sys::vga::set_font(&font);
                    } else {
                        eprintln!("Could not parse font file");
                        return usr::shell::ExitCode::CommandError;
                    }
                }
            } else if args.len() == 4 && args[2] == "palette" {
                if let Ok(csv) = fs::read_to_string(args[3]) {
                    if let Ok(palette) = palette::from_csv(&csv) {
                        sys::vga::set_palette(palette);
                        // TODO: Instead of calling a kernel function we could
                        // use the following ANSI OSC command to set a palette:
                        //     for (i, r, g, b) in palette.colors {
                        //         print!("\x1b]P{:x}{:x}{:x}{:x}", i, r, g, b);
                        //     }
                        // And "ESC]R" to reset a palette.
                    } else {
                        eprintln!("Could not parse palette file");
                        return usr::shell::ExitCode::CommandError;
                    }
                }
            } else {
                eprintln!("Invalid command");
                return usr::shell::ExitCode::CommandError;
            }
        },
        _ => {
            eprintln!("Invalid command");
            return usr::shell::ExitCode::CommandError;
        }
    }
    usr::shell::ExitCode::CommandSuccessful
}
