use crate::{sys, usr};
use crate::api::console::Style;
use crate::api::io;
use crate::sys::ata::Drive;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::vec;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 1 {
        return usage();
    }
    match args[1] {
        "format" if args.len() == 3 => format(args[2]),
        "erase" if args.len() == 3 => erase(args[2]),
        "usage" => usage(),
        "list" => list(),
        _ => help(),
    }
}

fn parse_disk_path(pathname: &str) -> Result<(u8, u8), String> {
    let path: Vec<_> = pathname.split('/').collect();
    if !pathname.starts_with("/dev/ata/") || path.len() != 5 {
        return Err(format!("Could not find disk at '{}'", pathname));
    }
    let bus = path[3].parse().or(Err("Could not parse <bus>".to_string()))?;
    let dsk = path[4].parse().or(Err("Could not parse <dsk>".to_string()))?;
    Ok((bus, dsk))
}

fn format(pathname: &str) -> usr::shell::ExitCode {
    match parse_disk_path(pathname) {
        Ok((bus, dsk)) => {
            sys::fs::mount_ata(bus, dsk);
            sys::fs::format_ata();
            println!("Disk successfully formatted");
            println!("MFS is now mounted to '/'");
            usr::shell::ExitCode::CommandSuccessful
        }
        Err(msg) => {
            error!("{}", msg);
            usr::shell::ExitCode::CommandError
        }
    }
}

fn erase(pathname: &str) -> usr::shell::ExitCode {
    match parse_disk_path(pathname) {
        Ok((bus, dsk)) => {
            if let Some(drive) = Drive::open(bus, dsk) {
                print!("Proceed? [y/N] ");
                if io::stdin().read_line().trim() == "y" {
                    println!();

                    let n = drive.block_count();
                    let buf = vec![0; drive.block_size() as usize];
                    print!("\x1b[?25l"); // Disable cursor
                    for i in 0..n {
                        if sys::console::end_of_text() {
                            println!();
                            print!("\x1b[?25h"); // Enable cursor
                            return usr::shell::ExitCode::CommandError;
                        }
                        print!("\x1b[2K\x1b[1G");
                        print!("Erasing block {}/{}", i, n);
                        // TODO: Implement drive.write(block, buf)
                        sys::ata::write(bus, dsk, i, &buf).ok();
                    }
                    println!();
                    print!("\x1b[?25h"); // Enable cursor
                }
            }
            usr::shell::ExitCode::CommandSuccessful
        }
        Err(msg) => {
            error!("{}", msg);
            usr::shell::ExitCode::CommandError
        }
    }
}

fn list() -> usr::shell::ExitCode {
    println!("Path            Name (Size)");
    for drive in sys::ata::list() {
        println!("/dev/ata/{}/{}    {}", drive.bus, drive.dsk, drive);
    }
    usr::shell::ExitCode::CommandSuccessful
}

fn usage() -> usr::shell::ExitCode {
    let size = sys::fs::disk_size();
    let used = sys::fs::disk_used();
    let free = size - used;

    let width = size.to_string().len();
    let color = Style::color("LightCyan");
    let reset = Style::reset();
    println!("{}size:{} {:width$} bytes", color, reset, size, width = width);
    println!("{}used:{} {:width$} bytes", color, reset, used, width = width);
    println!("{}free:{} {:width$} bytes", color, reset, free, width = width);
    usr::shell::ExitCode::CommandSuccessful
}

fn help() -> usr::shell::ExitCode {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} disk {}<command>{}", csi_title, csi_reset, csi_option, csi_reset);
    println!();
    println!("{}Commands:{}", csi_title, csi_reset);
    println!("  {}list{}             List detected disks", csi_option, csi_reset);
    println!("  {}usage{}            List disk usage", csi_option, csi_reset);
    println!("  {}format <path>{}    Format disk", csi_option, csi_reset);
    println!("  {}erase <path>{}     Erase disk", csi_option, csi_reset);
    usr::shell::ExitCode::CommandSuccessful
}
