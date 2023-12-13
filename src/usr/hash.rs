use crate::api;
use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::syscall;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::str;
use sha2::{Digest, Sha256};

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut i = 1;
    let n = args.len();
    let mut paths = Vec::new();
    let mut full = false;
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                help();
                return Ok(());
            }
            "-f" | "--full" => {
                full = true;
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
        if let Err(code) = print_hash(path, full) {
            return Err(code);
        }
    }
    Ok(())
}

pub fn print_hash(path: &str, full: bool) -> Result<(), ExitCode> {
    let n = if full { 32 } else { 16 };
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
        } else {
            error!("Could not read '{}'", path);
            Err(ExitCode::Failure)
        }
    } else {
        error!("Could not find file '{}'", path);
        Err(ExitCode::Failure)
    }
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} hash {}<file>{}", csi_title, csi_reset, csi_option, csi_reset);
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!("  {0}-f{1}, {0}--full{1}     Show full hash", csi_option, csi_reset);
}
