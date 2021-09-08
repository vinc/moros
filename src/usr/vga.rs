use crate::{api, sys, usr};
use crate::api::vga::palette;
use crate::api::fs::FileIO;
use alloc::vec;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 1 {
        println!("Usage: vga <command>");
        return usr::shell::ExitCode::CommandError;
    }
    match args[1] {
        "set" => {
            if args.len() == 4 && args[2] == "font" {
                if let Some(mut file) = sys::fs::File::open(args[3]) {
                    let mut buf = vec![0; file.size()];
                    file.read(&mut buf).ok();
                    if let Ok(font) = api::font::from_bytes(&buf) {
                        sys::vga::set_font(&font);
                    } else {
                        println!("Could not parse font file");
                        return usr::shell::ExitCode::CommandError;
                    }
                }
            } else if args.len() == 4 && args[2] == "palette" {
                if let Some(mut file) = sys::fs::File::open(args[3]) {
                    if let Ok(palette) = palette::from_csv(&file.read_to_string()) {
                        sys::vga::set_palette(palette);
                        // TODO: Instead of calling a kernel function we could
                        // use the following ANSI OSC command to set a palette:
                        //     for (i, r, g, b) in palette.colors {
                        //         print!("\x1b]P{:x}{:x}{:x}{:x}", i, r, g, b);
                        //     }
                        // And "ESC]R" to reset a palette.
                    } else {
                        println!("Could not parse palette file");
                        return usr::shell::ExitCode::CommandError;
                    }
                }
            } else {
                println!("Invalid command");
                return usr::shell::ExitCode::CommandError;
            }
        },
        _ => {
            println!("Invalid command");
            return usr::shell::ExitCode::CommandError;
        }
    }
    usr::shell::ExitCode::CommandSuccessful
}
