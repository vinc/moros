use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::font::Font;
use crate::api::vga::palette;
use crate::sys;

// TODO: Remove this command when everything can be done from userspace
pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() == 1 {
        help();
        return Err(ExitCode::UsageError);
    }

    match args[1] {
        "-h" | "--help" => {
            help();
            Ok(())
        }
        "set" => {
            if args.len() == 4 && args[2] == "font" {
                if let Ok(buf) = fs::read_to_bytes(args[3]) {
                    if let Ok(font) = Font::from_bytes(&buf) {
                        sys::vga::set_font(&font);
                        Ok(())
                    } else {
                        error!("Could not parse font file");
                        Err(ExitCode::Failure)
                    }
                } else {
                    error!("Could not read font file");
                    Err(ExitCode::Failure)
                }
            } else if args.len() == 4 && args[2] == "palette" {
                warning!("Use ANSI OSC palette sequence");
                if let Ok(csv) = fs::read_to_string(args[3]) {
                    if let Ok(palette) = palette::from_csv(&csv) {
                        sys::vga::set_palette(palette);
                        Ok(())
                    } else {
                        error!("Could not parse palette file");
                        Err(ExitCode::Failure)
                    }
                } else {
                    error!("Could not read palette file");
                    Err(ExitCode::Failure)
                }
            } else {
                error!("Invalid command");
                Err(ExitCode::Failure)
            }
        }
        _ => {
            error!("Invalid command");
            Err(ExitCode::Failure)
        }
    }
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} vga {}<command>{1}",
        csi_title, csi_reset, csi_option
    );
    println!();
    println!("{}Commands:{}", csi_title, csi_reset);
    println!(
        "  {}set font <file>{}       Set VGA font",
        csi_option, csi_reset
    );
    println!(
        "  {}set palette <file>{}    Set VGA color palette",
        csi_option, csi_reset
    );
}
