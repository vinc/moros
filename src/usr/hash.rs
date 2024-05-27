use crate::api;
use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::syscall;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::str;
use sha2::{Digest, Sha256};

#[derive(Copy, Clone)]
struct Config {
    show_full_hash: bool,
    recursive_mode: bool,
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut i = 1;
    let n = args.len();
    let mut paths = Vec::new();
    let mut conf = Config {
        show_full_hash: false,
        recursive_mode: false,
    };
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                help();
                return Ok(());
            }
            "-f" | "--full" => {
                conf.show_full_hash = true;
            }
            "-r" | "--recursive" => {
                conf.recursive_mode = true;
            }
            arg => {
                if arg.starts_with('-') {
                    error!("Unknown option '{}'", arg);
                    return Err(ExitCode::UsageError);
                }
                paths.push(arg);
            }
        }
        i += 1;
    }

    paths.sort();
    for path in paths {
        let path = if path.len() > 1 {
            path.trim_end_matches('/')
        } else {
            path
        };
        print_hash(path , conf)?;
    }
    Ok(())
}

fn print_hash(path: &str, conf: Config) -> Result<(), ExitCode> {
    let n = if conf.show_full_hash { 32 } else { 16 };
    if let Some(info) = syscall::info(path) {
        if info.is_file() {
            if let Ok(bytes) = api::fs::read_to_bytes(path) {
                let mut hasher = Sha256::new();
                hasher.update(bytes);
                let res = hasher.finalize();
                let hex = res.iter().map(|byte|
                    format!("{:02X}", byte)
                ).take(n).collect::<Vec<String>>().join("");
                let pink = Style::color("Pink");
                let reset = Style::reset();
                println!("{}{}{} {}", pink, hex, reset, path);
                Ok(())
            } else {
                error!("Could not read '{}'", path);
                Err(ExitCode::Failure)
            }
        } else if conf.recursive_mode && info.is_dir() {
            if let Ok(entries) = api::fs::read_dir(path) {
                let mut fs: Vec<_> = entries.iter().map(|e| e.name()).collect();
                fs.sort();
                for f in fs.iter() {
                    let s = if path == "/" { "" } else { "/" };
                    let p = format!("{}{}{}", path, s, f);
                    print_hash(&p, conf)?;
                }
                Ok(())
            } else {
                error!("Could not read '{}'", path);
                Err(ExitCode::Failure)
            }
        } else {
            error!("Could not hash '{}'", path);
            Err(ExitCode::Failure)
        }
    } else {
        error!("Could not find file '{}'", path);
        Err(ExitCode::Failure)
    }
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} hash {}<options> <file>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!(
        "  {0}-f{1}, {0}--full{1}         Show full hash",
        csi_option, csi_reset
    );
    println!(
        "  {0}-r{1}, {0}--recursive{1}    Enable recursive mode",
        csi_option, csi_reset
    );
}
