use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;

use alloc::format;
use miniz_oxide::deflate::compress_to_vec_zlib as deflate;

const LEVEL: u8 = 9; // From 1 (fast) to 9 (best)

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 2 {
        help();
        return Err(ExitCode::UsageError);
    }
    if args[1] == "-h" || args[1] == "--help" {
        help();
        return Ok(());
    }

    let path = args[1];
    if fs::is_file(path) {
        if let Ok(bytes) = fs::read_to_bytes(path) {
            let buf = deflate(&bytes, LEVEL);
            let dest = format!("{}.z", path);
            if fs::write(&dest, &buf).is_ok() {
                if fs::delete(path).is_ok() {
                    Ok(())
                } else {
                    error!("Could not drop {:?}", path);
                    Err(ExitCode::Failure)
                }
            } else {
                error!("Could not deflate to {:?}", dest);
                Err(ExitCode::Failure)
            }
        } else {
            error!("Could not read {:?}", path);
            Err(ExitCode::Failure)
        }
    } else {
        error!("Could not open {:?}", path);
        Err(ExitCode::Failure)
    }
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} deflate {}<file>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
}
