use crate::{api, sys, usr};
use crate::api::console::Style;
use crate::api::vga::palette;
use crate::api::fs;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 1 {
        help();
        return usr::shell::ExitCode::CommandError;
    }

    match args[1] {
        "-h" | "--help" => {
            help()
        }
        "set" => {
            if args.len() == 4 && args[2] == "font" {
                if let Ok(buf) = fs::read_to_bytes(args[3]) {
                    if let Ok(font) = api::font::from_bytes(&buf) {
                        sys::vga::set_font(&font);
                        usr::shell::ExitCode::CommandSuccessful
                    } else {
                        error!("Could not parse font file");
                        usr::shell::ExitCode::CommandError
                    }
                } else {
                    error!("Could not read font file");
                    usr::shell::ExitCode::CommandError
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
                        usr::shell::ExitCode::CommandSuccessful
                    } else {
                        error!("Could not parse palette file");
                        usr::shell::ExitCode::CommandError
                    }
                } else {
                    error!("Could not read palette file");
                    usr::shell::ExitCode::CommandError
                }
            } else {
                error!("Invalid command");
                usr::shell::ExitCode::CommandError
            }
        }
        _ => {
            error!("Invalid command");
            usr::shell::ExitCode::CommandError
        }
    }
}

fn help() -> usr::shell::ExitCode {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} vga {}<command>{1}", csi_title, csi_reset, csi_option);
    println!();
    println!("{}Commands:{}", csi_title, csi_reset);
    println!("  {}set font <file>{}       Set VGA font", csi_option, csi_reset);
    println!("  {}set palette <file>{}    Set VGA color palette", csi_option, csi_reset);
    usr::shell::ExitCode::CommandSuccessful
}
