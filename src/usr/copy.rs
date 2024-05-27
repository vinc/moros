use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;

use alloc::format;
use alloc::string::{String, ToString};

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let n = args.len();
    for i in 1..n {
        match args[i] {
            "-h" | "--help" => {
                help();
                return Ok(());
            }
            _ => continue,
        }
    }
    if n != 3 {
        help();
        return Err(ExitCode::UsageError);
    }

    if args[2].is_empty() {
        error!("Could not write to ''");
        return Err(ExitCode::Failure);
    }

    let source = args[1];
    let dest = destination(args[1], args[2]);

    if fs::is_dir(source) {
        error!("Could not copy directory '{}'", source);
        return Err(ExitCode::Failure);
    }

    if let Ok(contents) = fs::read_to_bytes(source) {
        if fs::write(&dest, &contents).is_ok() {
            Ok(())
        } else {
            error!("Could not write to '{}'", dest);
            Err(ExitCode::Failure)
        }
    } else {
        error!("Could not read file '{}'", source);
        Err(ExitCode::Failure)
    }
}

fn destination(source: &str, dest: &str) -> String {
    debug_assert!(!dest.is_empty());
    let mut dest = dest.trim_end_matches('/').to_string();
    if dest.is_empty() || fs::is_dir(&dest) {
        let file = fs::filename(source);
        dest = format!("{}/{}", dest, file);
    }
    dest
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} copy {}<src> <dst>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
}

#[test_case]
fn test_destination() {
    use crate::{usr, sys};

    sys::fs::mount_mem();
    sys::fs::format_mem();
    usr::install::copy_files(false);

    assert_eq!(destination("foo.txt", "bar.txt"), "bar.txt");

    assert_eq!(destination("foo.txt", "/"), "/foo.txt");
    assert_eq!(destination("foo.txt", "/tmp"), "/tmp/foo.txt");
    assert_eq!(destination("foo.txt", "/tmp/"), "/tmp/foo.txt");

    assert_eq!(destination("/usr/vinc/foo.txt", "/"), "/foo.txt");
    assert_eq!(destination("/usr/vinc/foo.txt", "/tmp"), "/tmp/foo.txt");
    assert_eq!(destination("/usr/vinc/foo.txt", "/tmp/"), "/tmp/foo.txt");

    sys::fs::dismount();
}
