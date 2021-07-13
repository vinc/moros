use crate::{kernel, print, user};
use crate::kernel::vga::Palette;
use alloc::vec::Vec;
use core::convert::TryInto;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() == 1 {
        print!("Usage: vga <command>\n");
        return user::shell::ExitCode::CommandError;
    }
    match args[1] {
        "set" => {
            if args.len() == 4 && args[2] == "font" {
                if let Some(file) = kernel::fs::File::open(args[3]) {
                    let size = file.size();
                    let mut buf = Vec::with_capacity(size);
                    buf.resize(size, 0);
                    file.read(&mut buf);
                    if let Ok(font) = kernel::fonts::from_bytes(&buf) {
                        kernel::vga::set_font(&font);
                    } else {
                        print!("Could not parse font file\n");
                        return user::shell::ExitCode::CommandError;
                    }
                }
            } else if args.len() == 4 && args[2] == "palette" {
                if let Some(file) = kernel::fs::File::open(args[3]) {
                    let mut colors = Vec::with_capacity(16);
                    for line in file.read_to_string().split("\n") {
                        let line = line.split("#").next().unwrap();
                        let color: Vec<u8> = line.split(",").filter_map(|value| {
                            let radix = if value.contains("0x") { 16 } else { 10 };
                            let value = value.trim().trim_start_matches("0x");
                            u8::from_str_radix(value, radix).ok()
                        }).collect();
                        if color.len() == 4 {
                            colors.push((color[0], color[1], color[2], color[3]));
                        }
                    }
                    if let Ok(colors) = colors.try_into() {
                        let palette = Palette { colors };
                        kernel::vga::set_palette(palette);
                    } else {
                        print!("Could not parse palette file\n");
                        return user::shell::ExitCode::CommandError;
                    }
                }

            } else {
                print!("Invalid command\n");
                return user::shell::ExitCode::CommandError;
            }
        },
        _ => {
            print!("Invalid command\n");
            return user::shell::ExitCode::CommandError;
        }
    }
    user::shell::ExitCode::CommandSuccessful
}
