use crate::sys;
use crate::api::clock::DATE_TIME;
use crate::api::console::Style;
use crate::api::time;
use crate::api::fs;
use crate::api::fs::FileInfo;
use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::api::unit::SizeUnit;

use alloc::string::ToString;
use alloc::vec::Vec;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut path: &str = &sys::process::dir(); // TODO: use '.'
    let mut sort = "name";
    let mut hide_dot_files = true;
    let mut unit = SizeUnit::None;

    let n = args.len();
    for i in 1..n {
        match args[i] {
            "-h" | "--help" => return help(),
            "-a" | "--all"  => hide_dot_files = false,
            "-n" | "--name" => sort = "name",
            "-s" | "--size" => sort = "size",
            "-t" | "--time" => sort = "time",
            "-b" | "--binary-size" => unit = SizeUnit::Binary,
            "-d" | "--decimal-size" => unit = SizeUnit::Decimal,
            _ => path = args[i],
        }
    }

    // The commands `list /usr/alice/` and `list /usr/alice` are equivalent,
    // but `list /` should not be modified.
    if path.len() > 1 {
        path = path.trim_end_matches('/');
    }

    if let Some(info) = syscall::info(path) {
        if info.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                let mut files: Vec<_> = entries.iter().filter(|entry| {
                    !(entry.name().starts_with('.') && hide_dot_files)
                }).collect();

                match sort {
                    "name" => files.sort_by_key(|f| f.name()),
                    "size" => files.sort_by_key(|f| f.size()),
                    "time" => files.sort_by_key(|f| f.time()),
                    _ => {
                        error!("Invalid sort key '{}'", sort);
                        return Err(ExitCode::Failure);
                    }
                }

                let width = files.iter().fold(0, |max_len, file| {
                    let len = unit.format(file.size() as usize).len();
                    core::cmp::max(max_len, len)
                });

                for file in files {
                    print_file(file, width, unit.clone());
                }
                Ok(())
            } else {
                error!("Could not read directory '{}'", path);
                Err(ExitCode::Failure)
            }
        } else {
            print_file(&info, info.size().to_string().len(), unit);
            Ok(())
        }
    } else {
        error!("Could not find file or directory '{}'", path);
        Err(ExitCode::Failure)
    }
}

fn print_file(file: &FileInfo, width: usize, unit: SizeUnit) {
    let csi_dir_color = Style::color("LightCyan");
    let csi_dev_color = Style::color("Yellow");
    let csi_reset = Style::reset();

    let size = unit.format(file.size() as usize);
    let time = time::from_timestamp(file.time() as i64).format(DATE_TIME);
    let color = if file.is_dir() {
        csi_dir_color
    } else if file.is_device() {
        csi_dev_color
    } else {
        csi_reset
    };
    println!("{:>width$} {} {}{}{}", size, time, color, file.name(), csi_reset, width = width);
}

fn help() -> Result<(), ExitCode> {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} list {}<options> [<dir>]{}", csi_title, csi_reset, csi_option, csi_reset);
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!("  {0}-a{1},{0} --all{1}     Show dot files", csi_option, csi_reset);
    println!("  {0}-n{1},{0} --name{1}    Sort by name", csi_option, csi_reset);
    println!("  {0}-s{1},{0} --size{1}    Sort by size", csi_option, csi_reset);
    println!("  {0}-t{1},{0} --time{1}    Sort by time", csi_option, csi_reset);
    Ok(())
}
