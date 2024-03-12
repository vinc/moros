use crate::api::console::Style;
use crate::api::io;
use crate::api::process::ExitCode;
use crate::api::unit::SizeUnit;
use crate::sys;
use crate::sys::ata::Drive;
use crate::sys::console;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    match *args.get(1).unwrap_or(&"") {
        "f" | "format" if args.len() == 3 => format(args[2]),
        "e" | "erase" if args.len() == 3 => erase(args[2]),
        "u" | "usage" => usage(&args[2..]),
        "l" | "list" => list(),
        "-h" | "--help" => {
            help();
            Ok(())
        }
        _ => {
            help();
            Err(ExitCode::UsageError)
        }
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

fn format(pathname: &str) -> Result<(), ExitCode> {
    match parse_disk_path(pathname) {
        Ok((bus, dsk)) => {
            sys::fs::mount_ata(bus, dsk);
            sys::fs::format_ata();
            println!("Disk successfully formatted");
            println!("MFS is now mounted to '/'");
            Ok(())
        }
        Err(msg) => {
            error!("{}", msg);
            Err(ExitCode::Failure)
        }
    }
}

fn is_canceled() -> bool {
    console::end_of_text() || console::end_of_transmission()
}

fn erase(pathname: &str) -> Result<(), ExitCode> {
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
                        if is_canceled() {
                            println!();
                            print!("\x1b[?25h"); // Enable cursor
                            return Err(ExitCode::Failure);
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
            Ok(())
        }
        Err(msg) => {
            error!("{}", msg);
            Err(ExitCode::Failure)
        }
    }
}

fn list() -> Result<(), ExitCode> {
    println!("Path            Name (Size)");
    for drive in sys::ata::list() {
        println!("/dev/ata/{}/{}    {}", drive.bus, drive.dsk, drive);
    }
    Ok(())
}

fn usage(args: &[&str]) -> Result<(), ExitCode> {
    let mut unit = SizeUnit::None;
    for arg in args {
        match *arg {
            "-b" | "--binary-size" => {
                unit = SizeUnit::Binary;
            }
            "-d" | "--decimal-size" => {
                unit = SizeUnit::Decimal;
            }
            "-h" | "--help" => {
                help_usage();
                return Ok(());
            }
            _ => {
                help_usage();
                return Err(ExitCode::Failure);
            }
        }
    }
    let size = sys::fs::disk_size();
    let used = sys::fs::disk_used();
    let free = size - used;
    let width = [size, used, free].iter().fold(0, |acc, num|
        core::cmp::max(acc, unit.format(*num).len())
    );
    let color = Style::color("LightCyan");
    let reset = Style::reset();
    println!(
        "{}size:{} {:>width$}",
        color,
        reset,
        unit.format(size),
        width = width
    );
    println!(
        "{}used:{} {:>width$}",
        color,
        reset,
        unit.format(used),
        width = width
    );
    println!(
        "{}free:{} {:>width$}",
        color,
        reset,
        unit.format(free),
        width = width
    );
    Ok(())
}

fn help_usage() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} disk usage {}<options>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!(
        "  {0}-b{1}, {0}--binary-size{1}   Use binary size",
        csi_option, csi_reset
    );
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} disk {}<command>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
    println!();
    println!("{}Commands:{}", csi_title, csi_reset);
    println!(
        "  {}erase <path>{}    Erase disk", csi_option, csi_reset
    );
    println!(
        "  {}format <path>{}   Format disk", csi_option, csi_reset
    );
    println!(
        "  {}list{}            List detected disks", csi_option, csi_reset
    );
    println!(
        "  {}usage{}           List disk usage", csi_option, csi_reset
    );
}
