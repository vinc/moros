use crate::{sys, usr};
use crate::api::console::Style;
use crate::api::fs;
use crate::api::io;
use crate::api::syscall;
use alloc::string::String;

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Welcome to MOROS v{} installation program!{}", csi_color, env!("CARGO_PKG_VERSION"), csi_reset);
    println!();

    print!("Proceed? [y/N] ");
    if io::stdin().read_line().trim() == "y" {
        println!();

        if !sys::fs::is_mounted() {
            println!("{}Listing disks ...{}", csi_color, csi_reset);
            usr::disk::main(&["disk", "list"]);
            println!();

            println!("{}Formatting disk ...{}", csi_color, csi_reset);
            print!("Enter path of disk to format: ");
            let pathname = io::stdin().read_line();
            let res = usr::disk::main(&["disk", "format", pathname.trim_end()]);
            if res == usr::shell::ExitCode::CommandError {
                return res;
            }
            println!();
        }

        println!("{}Populating filesystem...{}", csi_color, csi_reset);
        create_dir("/bin"); // Binaries
        create_dir("/dev"); // Devices
        create_dir("/ini"); // Initializers
        create_dir("/lib"); // Libraries
        create_dir("/net"); // Network
        create_dir("/src"); // Sources
        create_dir("/tmp"); // Temporaries
        create_dir("/usr"); // User directories
        create_dir("/var"); // Variables

        create_dir("/dev/clk"); // Clocks
        let pathname = "/dev/console";
        if syscall::stat(pathname).is_none() {
            if fs::create_device(pathname, sys::fs::DeviceType::Console).is_some() {
                println!("Created '{}'", pathname);
            }
        }
        let pathname = "/dev/random";
        if syscall::stat(pathname).is_none() {
            if fs::create_device(pathname, sys::fs::DeviceType::Random).is_some() {
                println!("Created '{}'", pathname);
            }
        }

        copy_file("/ini/boot.sh", include_bytes!("../../dsk/ini/boot.sh"));
        copy_file("/ini/banner.txt", include_bytes!("../../dsk/ini/banner.txt"));
        copy_file("/ini/version.txt", include_bytes!("../../dsk/ini/version.txt"));
        copy_file("/ini/palette.csv", include_bytes!("../../dsk/ini/palette.csv"));

        create_dir("/ini/fonts");
        copy_file("/ini/fonts/lat15-terminus-8x16.psf", include_bytes!("../../dsk/ini/fonts/lat15-terminus-8x16.psf"));
        copy_file("/ini/fonts/zap-light-8x16.psf", include_bytes!("../../dsk/ini/fonts/zap-light-8x16.psf"));
        copy_file("/ini/fonts/zap-vga-8x16.psf", include_bytes!("../../dsk/ini/fonts/zap-vga-8x16.psf"));

        copy_file("/tmp/alice.txt", include_bytes!("../../dsk/tmp/alice.txt"));
        copy_file("/tmp/fibonacci.lisp", include_bytes!("../../dsk/tmp/fibonacci.lisp"));

        create_dir("/tmp/beep");
        copy_file("/tmp/beep/tetris.sh", include_bytes!("../../dsk/tmp/beep/tetris.sh"));
        copy_file("/tmp/beep/starwars.sh", include_bytes!("../../dsk/tmp/beep/starwars.sh"));
        copy_file("/tmp/beep/mario.sh", include_bytes!("../../dsk/tmp/beep/mario.sh"));

        if sys::process::user().is_none() {
            println!();
            println!("{}Creating user...{}", csi_color, csi_reset);
            let res = usr::user::main(&["user", "create"]);
            if res == usr::shell::ExitCode::CommandError {
                return res;
            }
        }

        println!();
        println!("{}Installation successful!{}", csi_color, csi_reset);
        println!();
        println!("Exit console or reboot to apply changes");
    }

    usr::shell::ExitCode::CommandSuccessful
}

fn create_dir(pathname: &str) {
    if sys::fs::Dir::create(pathname).is_some() {
        println!("Created '{}'", pathname);
    }
}

fn copy_file(pathname: &str, buf: &[u8]) {
    if syscall::stat(pathname).is_some() {
        return;
    }
    if pathname.ends_with(".txt") {
        if let Ok(text) = String::from_utf8(buf.to_vec()) {
            let text = text.replace("{x.x.x}", env!("CARGO_PKG_VERSION"));
            fs::write(pathname, text.as_bytes()).ok();
        } else {
            fs::write(pathname, buf).ok();
        }
    } else {
        fs::write(pathname, buf).ok();
    }
    // TODO: add File::write_all to split buf if needed
    println!("Copied '{}'", pathname);
}
