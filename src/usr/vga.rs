use crate::{api, sys, usr};
use crate::api::vga::palette;
use alloc::vec;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 1 {
        print!("Usage: vga <command>\n");
        return usr::shell::ExitCode::CommandError;
    }
    match args[1] {
        "set" => {
            if args.len() == 4 && args[2] == "font" {
                if let Some(mut file) = sys::fs::File::open(args[3]) {
                    let mut buf = vec![0; file.size()];
                    file.read(&mut buf);
                    if let Ok(font) = api::font::from_bytes(&buf) {
                        sys::vga::set_font(&font);
                    } else {
                        print!("Could not parse font file\n");
                        return usr::shell::ExitCode::CommandError;
                    }
                }
            } else if args.len() == 4 && args[2] == "palette" {
                if let Some(mut file) = sys::fs::File::open(args[3]) {
                    if let Ok(palette) = palette::from_csv(&file.read_to_string()) {
                        sys::vga::set_palette(palette);
                    } else {
                        print!("Could not parse palette file\n");
                        return usr::shell::ExitCode::CommandError;
                    }
                }
            } else {
                print!("Invalid command\n");
                return usr::shell::ExitCode::CommandError;
            }
        },
        _ => {
            print!("Invalid command\n");
            return usr::shell::ExitCode::CommandError;
        }
    }
    usr::shell::ExitCode::CommandSuccessful
}
