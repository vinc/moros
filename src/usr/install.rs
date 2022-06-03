use crate::{api, sys, usr};
use crate::api::console::Style;
use crate::api::fs;
use crate::api::fs::DeviceType;
use crate::api::io;
use crate::api::syscall;
use alloc::string::String;

pub fn copy_files(verbose: bool) {
    create_dir("/bin", verbose); // Binaries
    create_dir("/dev", verbose); // Devices
    create_dir("/ini", verbose); // Initializers
    create_dir("/lib", verbose); // Libraries
    create_dir("/net", verbose); // Network
    create_dir("/src", verbose); // Sources
    create_dir("/tmp", verbose); // Temporaries
    create_dir("/usr", verbose); // User directories
    create_dir("/var", verbose); // Variables

    copy_file("/bin/hello", include_bytes!("../../dsk/bin/hello"), verbose);
    copy_file("/bin/sleep", include_bytes!("../../dsk/bin/sleep"), verbose);

    create_dir("/dev/clk", verbose); // Clocks
    create_dev("/dev/clk/uptime", DeviceType::File, verbose); // TODO
    create_dev("/dev/clk/realtime", DeviceType::File, verbose); // TODO
    create_dev("/dev/rtc", DeviceType::File, verbose); // TODO
    create_dev("/dev/null", DeviceType::Null, verbose);
    create_dev("/dev/random", DeviceType::Random, verbose);
    create_dev("/dev/console", DeviceType::Console, verbose);

    copy_file("/ini/boot.sh", include_bytes!("../../dsk/ini/boot.sh"), verbose);
    copy_file("/ini/banner.txt", include_bytes!("../../dsk/ini/banner.txt"), verbose);
    copy_file("/ini/version.txt", include_bytes!("../../dsk/ini/version.txt"), verbose);
    copy_file("/ini/palette.csv", include_bytes!("../../dsk/ini/palette.csv"), verbose);

    create_dir("/ini/lisp", verbose);
    copy_file("/ini/lisp/core.lsp", include_bytes!("../../dsk/ini/lisp/core.lsp"), verbose);

    create_dir("/ini/fonts", verbose);
    copy_file("/ini/fonts/lat15-terminus-8x16.psf", include_bytes!("../../dsk/ini/fonts/lat15-terminus-8x16.psf"), verbose);
    copy_file("/ini/fonts/zap-light-8x16.psf", include_bytes!("../../dsk/ini/fonts/zap-light-8x16.psf"), verbose);
    copy_file("/ini/fonts/zap-vga-8x16.psf", include_bytes!("../../dsk/ini/fonts/zap-vga-8x16.psf"), verbose);

    copy_file("/tmp/alice.txt", include_bytes!("../../dsk/tmp/alice.txt"), verbose);

    create_dir("/tmp/lisp", verbose);
    copy_file("/tmp/lisp/factorial.lsp", include_bytes!("../../dsk/tmp/lisp/factorial.lsp"), verbose);
    copy_file("/tmp/lisp/fibonacci.lsp", include_bytes!("../../dsk/tmp/lisp/fibonacci.lsp"), verbose);

    create_dir("/tmp/beep", verbose);
    copy_file("/tmp/beep/tetris.sh", include_bytes!("../../dsk/tmp/beep/tetris.sh"), verbose);
    copy_file("/tmp/beep/starwars.sh", include_bytes!("../../dsk/tmp/beep/starwars.sh"), verbose);
    copy_file("/tmp/beep/mario.sh", include_bytes!("../../dsk/tmp/beep/mario.sh"), verbose);
}

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
        let verbose = true;
        copy_files(verbose);

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

fn create_dir(pathname: &str, verbose: bool) {
    if syscall::info(pathname).is_none() {
        if let Some(handle) = api::fs::create_dir(pathname) {
            syscall::close(handle);
            if verbose {
                println!("Created '{}'", pathname);
            }
        }
    }
}

fn create_dev(pathname: &str, dev: DeviceType, verbose: bool) {
    if syscall::info(pathname).is_none() {
        if let Some(handle) = fs::create_device(pathname, dev) {
            syscall::close(handle);
            if verbose {
                println!("Created '{}'", pathname);
            }
        }
    }
}

fn copy_file(pathname: &str, buf: &[u8], verbose: bool) {
    if fs::exists(pathname) {
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
    if verbose {
        println!("Copied '{}'", pathname);
    }
}
