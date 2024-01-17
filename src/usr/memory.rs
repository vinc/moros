use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::unit::SizeUnit;
use crate::sys;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    match *args.get(1).unwrap_or(&"") {
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
    let size = sys::allocator::memory_size();
    let used = sys::allocator::memory_used();
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
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} memory {}<command>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
    println!();
    println!("{}Commands:{}", csi_title, csi_reset);
    println!("  {}usage{}     List memory usage", csi_option, csi_reset);
    println!("  {}format{}    Format RAM disk", csi_option, csi_reset);
}
