use crate::{sys, usr};
use crate::api::console::Style;
use alloc::string::ToString;
use alloc::vec::Vec;
use time::OffsetDateTime;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    let mut path: &str = &sys::process::dir();
    let mut sort = "name";

    let mut i = 1;
    let n = args.len();
    while i < n {
        match args[i] {
            "--sort" => {
                if i + 1 < n {
                    sort = args[i + 1];
                    i += 1;
                } else {
                    eprintln!("Missing sort key");
                    return usr::shell::ExitCode::CommandError;
                }
            },
            "-t" => sort = "time",
            "-s" => sort = "size",
            "-n" => sort = "name",
            _ => path = args[i],
        }
        i += 1;
    }

    // The commands `list /usr/alice/` and `list /usr/alice` are equivalent,
    // but `list /` should not be modified.
    if path.len() > 1 {
        path = path.trim_end_matches('/');
    }

    if let Some(dir) = sys::fs::Dir::open(path) {
        let mut files: Vec<_> = dir.entries().collect();

        match sort {
            "name" => files.sort_by_key(|f| f.name()),
            "size" => files.sort_by_key(|f| f.size()),
            "time" => files.sort_by_key(|f| f.time()),
            _ => {
                eprintln!("Invalid sort key '{}'", sort);
                return usr::shell::ExitCode::CommandError;
            }
        }

        let mut max_size = 0;
        for file in &files {
            max_size = core::cmp::max(max_size, file.size());
        }
        let width = max_size.to_string().len();

        let csi_dir_color = Style::color("Blue");
        let csi_dev_color = Style::color("Yellow");
        let csi_reset = Style::reset();

        for file in files {
            let date = OffsetDateTime::from_unix_timestamp(file.time() as i64);
            let color = if file.is_dir() {
                csi_dir_color
            } else if file.is_device() {
                csi_dev_color
            } else {
                csi_reset
            };
            println!("{:width$} {} {}{}{}", file.size(), date.format("%F %H:%M:%S"), color, file.name(), csi_reset, width = width);
        }
        usr::shell::ExitCode::CommandSuccessful
    } else {
        eprintln!("Dir not found '{}'", path);
        usr::shell::ExitCode::CommandError
    }
}
