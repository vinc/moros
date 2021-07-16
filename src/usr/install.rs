use crate::{sys, usr, print};
use crate::sys::console::Style;
use alloc::string::String;

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    print!("{}Welcome to MOROS v{} installation program!{}\n", csi_color, env!("CARGO_PKG_VERSION"), csi_reset);
    print!("\n");

    print!("Proceed? [y/N] ");
    if sys::console::get_line().trim() == "y" {
        print!("\n");

        if !sys::fs::is_mounted() {
            print!("{}Listing disks ...{}\n", csi_color, csi_reset);
            usr::disk::main(&["disk", "list"]);
            print!("\n");

            print!("{}Formatting disk ...{}\n", csi_color, csi_reset);
            print!("Enter path of disk to format: ");
            let pathname = sys::console::get_line();
            let res = usr::disk::main(&["disk", "format", pathname.trim_end()]);
            if res == usr::shell::ExitCode::CommandError {
                return res;
            }
            print!("\n");
        }

        print!("{}Populating filesystem...{}\n", csi_color, csi_reset);
        create_dir("/bin"); // Binaries
        create_dir("/dev"); // Devices
        create_dir("/ini"); // Initializers
        create_dir("/lib"); // Libraries
        create_dir("/net"); // Network
        create_dir("/src"); // Sources
        create_dir("/tmp"); // Temporaries
        create_dir("/usr"); // User directories
        create_dir("/var"); // Variables

        copy_file("/ini/boot.sh", include_bytes!("../../dsk/ini/boot.sh"));
        copy_file("/ini/banner.txt", include_bytes!("../../dsk/ini/banner.txt"));
        copy_file("/ini/version.txt", include_bytes!("../../dsk/ini/version.txt"));
        copy_file("/ini/palette.csv", include_bytes!("../../dsk/ini/palette.csv"));
        copy_file("/tmp/alice.txt", include_bytes!("../../dsk/tmp/alice.txt"));

        create_dir("/ini/fonts");
        copy_file("/ini/fonts/lat15-terminus-8x16.psf", include_bytes!("../../dsk/ini/fonts/lat15-terminus-8x16.psf"));
        copy_file("/ini/fonts/zap-light-8x16.psf", include_bytes!("../../dsk/ini/fonts/zap-light-8x16.psf"));
        copy_file("/ini/fonts/zap-vga-8x16.psf", include_bytes!("../../dsk/ini/fonts/zap-vga-8x16.psf"));

        if sys::process::user().is_none() {
            print!("\n");
            print!("{}Creating user...{}\n", csi_color, csi_reset);
            let res = usr::user::main(&["user", "create"]);
            if res == usr::shell::ExitCode::CommandError {
                return res;
            }
        }

        print!("\n");
        print!("{}Installation successful!{}\n", csi_color, csi_reset);
        print!("\n");
        print!("Exit console or reboot to apply changes\n");
    }

    usr::shell::ExitCode::CommandSuccessful
}

fn create_dir(pathname: &str) {
    if sys::fs::Dir::create(pathname).is_some() {
        print!("Created '{}'\n", pathname);
    }
}

fn copy_file(pathname: &str, buf: &[u8]) {
    if sys::fs::File::open(pathname).is_some() {
        return;
    }
    if let Some(mut file) = sys::fs::File::create(pathname) {
        if pathname.ends_with(".txt") {
            if let Ok(text) = String::from_utf8(buf.to_vec()) {
                let text = text.replace("{x.x.x}", env!("CARGO_PKG_VERSION"));
                file.write(&text.as_bytes()).unwrap();
            } else {
                file.write(buf).unwrap();
            }
        } else {
            file.write(buf).unwrap();
        }
        print!("Copied '{}'\n", pathname);
    }
}
