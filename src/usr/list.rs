use crate::sys;
use crate::api::console::Style;
use crate::api::time;
use crate::api::fs;
use crate::api::syscall;
use crate::api::fs::FileInfo;

use alloc::string::ToString;
use alloc::vec::Vec;

pub fn main(args: &[&str]) -> Result<(), usize> {
    let mut path: &str = &sys::process::dir(); // TODO: use '.'
    let mut sort = "name";
    let mut hide_dot_files = true;

    let n = args.len();
    for i in 1..n {
        match args[i] {
            "-h" | "--help" => return help(),
            "-a" | "--all"  => hide_dot_files = false,
            "-n" | "--name" => sort = "name",
            "-s" | "--size" => sort = "size",
            "-t" | "--time" => sort = "time",
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
                        return Err(1);
                    }
                }

                let mut max_size = 0;
                for file in &files {
                    max_size = core::cmp::max(max_size, file.size());
                }
                let width = max_size.to_string().len();


                for file in files {
                    print_file(file, width);
                }
                Ok(())
            } else {
                error!("Could not read directory '{}'", path);
                Err(1)
            }
        } else {
            print_file(&info, info.size().to_string().len());
            Ok(())
        }
    } else {
        error!("Could not find file or directory '{}'", path);
        Err(1)
    }
}

fn print_file(file: &FileInfo, width: usize) {
    let csi_dir_color = Style::color("LightCyan");
    let csi_dev_color = Style::color("Yellow");
    let csi_reset = Style::reset();

    let date = time::from_timestamp(file.time() as i64);
    let color = if file.is_dir() {
        csi_dir_color
    } else if file.is_device() {
        csi_dev_color
    } else {
        csi_reset
    };
    println!("{:width$} {} {}{}{}", file.size(), date.format("%F %H:%M:%S"), color, file.name(), csi_reset, width = width);
}

fn help() -> Result<(), usize> {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} list {}<options>{}", csi_title, csi_reset, csi_option, csi_reset);
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!("  {0}-a{1},{0} --all{1}     Show dot files", csi_option, csi_reset);
    println!("  {0}-n{1},{0} --name{1}    Sort by name", csi_option, csi_reset);
    println!("  {0}-s{1},{0} --size{1}    Sort by size", csi_option, csi_reset);
    println!("  {0}-t{1},{0} --time{1}    Sort by time", csi_option, csi_reset);
    Ok(())
}
