use crate::{api, sys, usr};
use crate::api::console::Style;
use crate::api::fs;
use crate::api::fs::DeviceType;
use crate::api::io;
use crate::api::process::ExitCode;
use crate::api::syscall;

use alloc::format;
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

    copy_file("/bin/clear", include_bytes!("../../dsk/bin/clear"), verbose);
    copy_file("/bin/halt", include_bytes!("../../dsk/bin/halt"), verbose);
    copy_file("/bin/hello", include_bytes!("../../dsk/bin/hello"), verbose);
    copy_file("/bin/print", include_bytes!("../../dsk/bin/print"), verbose);
    copy_file("/bin/reboot", include_bytes!("../../dsk/bin/reboot"), verbose);
    copy_file("/bin/sleep", include_bytes!("../../dsk/bin/sleep"), verbose);

    create_dir("/dev/clk", verbose); // Clock
    create_dev("/dev/clk/uptime", DeviceType::Uptime, verbose);
    create_dev("/dev/clk/realtime", DeviceType::Realtime, verbose);
    create_dev("/dev/rtc", DeviceType::RTC, verbose);
    create_dev("/dev/null", DeviceType::Null, verbose);
    create_dev("/dev/random", DeviceType::Random, verbose);
    create_dev("/dev/console", DeviceType::Console, verbose);
    create_dir("/dev/net", verbose); // Network
    create_dev("/dev/net/tcp", DeviceType::TcpSocket, verbose);
    create_dev("/dev/net/udp", DeviceType::UdpSocket, verbose);

    copy_file("/ini/banner.txt", include_bytes!("../../dsk/ini/banner.txt"), verbose);
    copy_file("/ini/boot.sh", include_bytes!("../../dsk/ini/boot.sh"), verbose);
    copy_file("/ini/shell.sh", include_bytes!("../../dsk/ini/shell.sh"), verbose);
    copy_file("/ini/version.txt", include_bytes!("../../dsk/ini/version.txt"), verbose);

    create_dir("/ini/palettes", verbose);
    copy_file("/ini/palettes/gruvbox-dark.csv", include_bytes!("../../dsk/ini/palettes/gruvbox-dark.csv"), verbose);
    copy_file("/ini/palettes/gruvbox-light.csv", include_bytes!("../../dsk/ini/palettes/gruvbox-light.csv"), verbose);

    create_dir("/ini/fonts", verbose);
    //copy_file("/ini/fonts/lat15-terminus-8x16.psf", include_bytes!("../../dsk/ini/fonts/lat15-terminus-8x16.psf"), verbose);
    copy_file("/ini/fonts/zap-light-8x16.psf", include_bytes!("../../dsk/ini/fonts/zap-light-8x16.psf"), verbose);
    copy_file("/ini/fonts/zap-vga-8x16.psf", include_bytes!("../../dsk/ini/fonts/zap-vga-8x16.psf"), verbose);

    create_dir("/lib/lisp", verbose);
    copy_file("/lib/lisp/alias.lsp", include_bytes!("../../dsk/lib/lisp/alias.lsp"), verbose);
    copy_file("/lib/lisp/core.lsp", include_bytes!("../../dsk/lib/lisp/core.lsp"), verbose);
    copy_file("/lib/lisp/file.lsp", include_bytes!("../../dsk/lib/lisp/file.lsp"), verbose);
    //copy_file("/lib/lisp/legacy.lsp", include_bytes!("../../dsk/lib/lisp/legacy.lsp"), verbose);

    copy_file("/tmp/alice.txt", include_bytes!("../../dsk/tmp/alice.txt"), verbose);
    copy_file("/tmp/machines.txt", include_bytes!("../../dsk/tmp/machines.txt"), verbose);

    create_dir("/tmp/lisp", verbose);
    copy_file("/tmp/lisp/colors.lsp", include_bytes!("../../dsk/tmp/lisp/colors.lsp"), verbose);
    copy_file("/tmp/lisp/factorial.lsp", include_bytes!("../../dsk/tmp/lisp/factorial.lsp"), verbose);
    //copy_file("/tmp/lisp/fetch.lsp", include_bytes!("../../dsk/tmp/lisp/ntp.lsp"), verbose);
    copy_file("/tmp/lisp/fibonacci.lsp", include_bytes!("../../dsk/tmp/lisp/fibonacci.lsp"), verbose);
    copy_file("/tmp/lisp/geotime.lsp", include_bytes!("../../dsk/tmp/lisp/geotime.lsp"), verbose);
    //copy_file("/tmp/lisp/ntp.lsp", include_bytes!("../../dsk/tmp/lisp/ntp.lsp"), verbose);
    copy_file("/tmp/lisp/pi.lsp", include_bytes!("../../dsk/tmp/lisp/pi.lsp"), verbose);
    copy_file("/tmp/lisp/sum.lsp", include_bytes!("../../dsk/tmp/lisp/sum.lsp"), verbose);

    create_dir("/tmp/life", verbose);
    copy_file("/tmp/life/centinal.cells", include_bytes!("../../dsk/tmp/life/centinal.cells"), verbose);
    copy_file("/tmp/life/flower-of-eden.cells", include_bytes!("../../dsk/tmp/life/flower-of-eden.cells"), verbose);
    copy_file("/tmp/life/garden-of-eden.cells", include_bytes!("../../dsk/tmp/life/garden-of-eden.cells"), verbose);
    copy_file("/tmp/life/glider-gun.cells", include_bytes!("../../dsk/tmp/life/glider-gun.cells"), verbose);
    copy_file("/tmp/life/pentadecathlon.cells", include_bytes!("../../dsk/tmp/life/pentadecathlon.cells"), verbose);
    copy_file("/tmp/life/queen-bee-shuttle.cells", include_bytes!("../../dsk/tmp/life/queen-bee-shuttle.cells"), verbose);
    copy_file("/tmp/life/ship-in-a-bottle.cells", include_bytes!("../../dsk/tmp/life/ship-in-a-bottle.cells"), verbose);
    copy_file("/tmp/life/thunderbird.cells", include_bytes!("../../dsk/tmp/life/thunderbird.cells"), verbose);
    copy_file("/tmp/life/wing.cells", include_bytes!("../../dsk/tmp/life/wing.cells"), verbose);

    create_dir("/tmp/beep", verbose);
    copy_file("/tmp/beep/tetris.sh", include_bytes!("../../dsk/tmp/beep/tetris.sh"), verbose);
    copy_file("/tmp/beep/starwars.sh", include_bytes!("../../dsk/tmp/beep/starwars.sh"), verbose);
    copy_file("/tmp/beep/mario.sh", include_bytes!("../../dsk/tmp/beep/mario.sh"), verbose);

    create_dir("/var/www", verbose);
    copy_file("/var/www/index.html", include_bytes!("../../dsk/var/www/index.html"), verbose);
    copy_file("/var/www/moros.png", include_bytes!("../../dsk/var/www/moros.png"), verbose);
    copy_file("/var/www/moros.css", include_bytes!("../../dsk/var/www/moros.css"), verbose);
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Welcome to MOROS v{} installation program!{}", csi_color, env!("CARGO_PKG_VERSION"), csi_reset);
    println!();

    let mut has_confirmed = false;
    for &arg in args {
        match arg {
            "-y" | "--yes" => has_confirmed = true,
            _ => continue
        }
    }
    if !has_confirmed {
        print!("Proceed? [y/N] ");
        has_confirmed = io::stdin().read_line().trim() == "y";
        println!();
    }

    if has_confirmed {
        if !sys::fs::is_mounted() {
            println!("{}Listing disks ...{}", csi_color, csi_reset);
            usr::shell::exec("disk list").ok();
            println!();

            println!("{}Formatting disk ...{}", csi_color, csi_reset);
            print!("Enter path of disk to format: ");
            let pathname = io::stdin().read_line();
            usr::shell::exec(&format!("disk format {}", pathname.trim_end()))?;
            println!();
        }

        println!("{}Populating filesystem...{}", csi_color, csi_reset);
        let verbose = true;
        copy_files(verbose);

        if sys::process::user().is_none() {
            println!();
            println!("{}Creating user...{}", csi_color, csi_reset);
            let res = usr::user::main(&["user", "create"]);
            if res == Err(ExitCode::Failure) {
                return res;
            }
        }

        println!();
        println!("{}Installation successful!{}", csi_color, csi_reset);
        println!();
        println!("Quit the console or reboot to apply changes");
    }

    Ok(())
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
