use crate::{kernel, print, user};
use alloc::vec::Vec;

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
