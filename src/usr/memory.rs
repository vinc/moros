use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::api::unit::SizeUnit;
use crate::sys;

use core::num::ParseIntError;
use x86_64::PhysAddr;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    match *args.get(1).unwrap_or(&"") {
        "d" | "dump" => dump(&args[2..]),
        "u" | "usage" => usage(&args[2..]),
        "f" | "format" => {
            sys::fs::mount_mem();
            sys::fs::format_mem();
            println!("Memory successfully formatted");
            println!("MFS is now mounted to '/'");
            Ok(())
        }
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

fn parse_usize(s: &str) -> Result<usize, ParseIntError> {
    if s.starts_with("0x") {
        usize::from_str_radix(&s[2..], 16)
    } else {
        usize::from_str_radix(s, 10)
    }
}

fn dump(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 2 {
        return Err(ExitCode::UsageError);
    }
    let addr = parse_usize(args[0]).unwrap();
    let size = parse_usize(args[1]).unwrap();
    let phys_addr = PhysAddr::new(addr as u64);
    let virt_addr = sys::mem::phys_to_virt(phys_addr);
    let buf = unsafe {
        core::slice::from_raw_parts(virt_addr.as_ptr(), size)
    };
    syscall::write(1, buf);
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
    let size = sys::mem::memory_size();
    let used = sys::mem::memory_used();
    let free = size - used;
    let width = [size, used, free].iter().fold(0, |acc, num|
        core::cmp::max(acc, unit.format(*num).len())
    );
    let color = Style::color("aqua");
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
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} memory usage {}<options>{}",
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
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} memory {}<command>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
    println!();
    println!("{}Commands:{}", csi_title, csi_reset);
    println!(
        "  {}dump <addr> <size>{}   Dump memory", csi_option, csi_reset
    );
    println!(
        "  {}format{}               Format RAM disk", csi_option, csi_reset
    );
    println!(
        "  {}usage{}                List memory usage", csi_option, csi_reset
    );
}
