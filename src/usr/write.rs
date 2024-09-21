use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::syscall;

use alloc::vec::Vec;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut opt = Vec::new();
    let mut parents = false;
    let mut dev = None;
    let mut i = 1;
    let n = args.len();
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                help();
                return Ok(());
            }
            "-p" | "--parents" => {
                parents = true
            }
            "-d" | "--dev" => {
                if i + 1 < n {
                    i += 1;
                    dev = Some(args[i]);
                } else {
                    error!("Missing device type");
                    return Err(ExitCode::UsageError);
                }
            }
            _ => opt.push(args[i]),
        }
        i += 1;
    }
    if opt.len() != 1 {
        help();
        return Err(ExitCode::UsageError);
    };
    let path = opt[0];

    if fs::exists(path) {
        error!("Could not write to '{}'", path);
        return Err(ExitCode::Failure);
    }

    if parents {
        create_parents(fs::dirname(path.trim_end_matches('/')));
    }

    // The command `write /usr/alice/` with a trailing slash will create
    // a directory, while the same command without a trailing slash will
    // create a file.

    let res = if path.ends_with('/') {
        let path = path.trim_end_matches('/');
        fs::create_dir(path)
    } else if let Some(name) = dev {
        fs::create_device(path, name)
    } else {
        fs::create_file(path)
    };

    if let Some(handle) = res {
        syscall::close(handle);
        Ok(())
    } else {
        error!("Could not write to '{}'", path);
        Err(ExitCode::Failure)
    }
}

fn create_parents(path: &str) {
    debug!("create_parents('{}')", path);
    if path.is_empty() || fs::exists(path) {
        return;
    }
    create_parents(fs::dirname(path));
    fs::create_dir(path);
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} write {}<options> <path>{1}",
        csi_title, csi_reset, csi_option
    );
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!(
        "  {0}-d{1}, {0}--device <type>{1}   {2}",
        csi_option, csi_reset, "Set device type"
    );
    println!(
        "  {0}-p{1}, {0}--parents{1}         {2}",
        csi_option, csi_reset, "Create parent directories as needed"
    );
    println!();
    println!("{}Paths:{}", csi_title, csi_reset);
    println!("  {0}<dir>/{1}     Write directory", csi_option, csi_reset);
    println!("  {0}<file>{1}     Write file", csi_option, csi_reset);
}
