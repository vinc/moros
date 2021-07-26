use crate::{sys, usr};
use crate::api::console::Style;
use alloc::string::ToString;
use alloc::vec::Vec;
use time::OffsetDateTime;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    let current_dir = sys::process::dir();
    let mut pathname = if args.len() == 2 && !args[1].is_empty() {
        args[1]
    } else {
        &current_dir
    };

    // The commands `list /usr/alice/` and `list /usr/alice` are equivalent,
    // but `list /` should not be modified.
    if pathname.len() > 1 {
        pathname = pathname.trim_end_matches('/');
    }

    if let Some(dir) = sys::fs::Dir::open(pathname) {
        let mut files: Vec<_> = dir.read().collect();

        files.sort_by_key(|f| f.name());

        //let w = files.map(|f| f.size()).to_string().len();
        let mut w = 0;
        for file in &files {
            w = core::cmp::max(w, file.size());
        }
        let w = w.to_string().len();

        let csi_color = Style::color("Blue");
        let csi_reset = Style::reset();

        for file in files {
            let date = OffsetDateTime::from_unix_timestamp(file.time() as i64);
            let color = if file.is_dir() { csi_color } else { csi_reset };
            println!("{:w$} {} {}{}{}", file.size(), date.format("%F %H:%M:%S"), color, file.name(), csi_reset, w = w);
        }
        usr::shell::ExitCode::CommandSuccessful
    } else {
        println!("Dir not found '{}'", pathname);
        usr::shell::ExitCode::CommandError
    }
}
